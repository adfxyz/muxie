use crate::browser::Browser;
use crate::paths::config_path;
use anyhow::{Context, Result, bail};
use freedesktop_desktop_entry::{Iter, default_paths};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub browsers: Vec<Browser>,

    #[serde(default)]
    pub notifications: Notifications,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Notifications {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_true")]
    pub redact_urls: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Notifications {
    fn default() -> Self {
        Notifications {
            enabled: true,
            redact_urls: true,
        }
    }
}

pub fn read_config() -> Result<Config> {
    let config_path = config_path();
    if !config_path.exists() {
        bail!(
            "Configuration not found. Please run 'muxie install' first to set up browser configuration at: {}",
            config_path.display()
        );
    }
    let config_text = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
    let config: Config = serde_yaml::from_str(&config_text)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;
    Ok(config)
}

// Dependency trait for reading configuration and a default impl.
pub(crate) trait ConfigReader {
    fn read_config(&self) -> Result<Config>;
}

#[derive(Default, Clone, Copy)]
pub(crate) struct DefaultConfigReader;

impl ConfigReader for DefaultConfigReader {
    fn read_config(&self) -> Result<Config> {
        read_config()
    }
}

pub fn installed_browsers() -> Vec<Browser> {
    Iter::new(default_paths())
        .filter_map(|path| {
            let entry_text = match std::fs::read_to_string(&path) {
                Ok(text) => text,
                Err(_) => return None,
            };
            let desktop_entry =
                match freedesktop_desktop_entry::DesktopEntry::decode(&path, &entry_text) {
                    Ok(entry) => entry,
                    Err(_) => return None,
                };
            let browser = Browser::from_desktop_entry(&desktop_entry);
            match browser {
                Some(browser) => {
                    if browser.name.contains("Muxie") {
                        return None;
                    }
                    Some(browser)
                }
                None => None,
            }
        })
        .collect()
}

pub fn ensure_config() -> Result<()> {
    let config_path = config_path();
    if !config_path.exists() {
        let config = Config {
            browsers: installed_browsers(),
            notifications: Notifications::default(),
        };
        let config_text =
            serde_yaml::to_string(&config).context("Failed to serialize default config")?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        std::fs::write(&config_path, config_text)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ValidationError {
    fn new(code: &str, message: impl Into<String>, path: impl Into<Option<String>>) -> Self {
        ValidationError {
            code: code.to_string(),
            message: message.into(),
            path: path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl Config {
    /// Validate semantic constraints. Does not perform I/O checks unless `strict` is true.
    pub fn validate(&self, strict: bool) -> ValidationResult {
        let mut errors: Vec<ValidationError> = Vec::new();

        // Browsers present
        if self.browsers.is_empty() {
            errors.push(ValidationError::new(
                "browsers.empty",
                "No browsers configured",
                Some("browsers".to_string()),
            ));
        }

        // Duplicate browser names, empty fields, and exec presence
        let mut names: HashSet<&str> = HashSet::new();
        for (i, b) in self.browsers.iter().enumerate() {
            let path = |field: &str| format!("browsers[{i}].{field}");
            if b.name.trim().is_empty() {
                errors.push(ValidationError::new(
                    "browser.name.empty",
                    "Browser name must not be empty",
                    Some(path("name")),
                ));
            }
            if !b.name.is_empty() && !names.insert(b.name.as_str()) {
                errors.push(ValidationError::new(
                    "browser.name.duplicate",
                    format!("Duplicate browser name: {}", b.name),
                    Some(path("name")),
                ));
            }
            if b.executable.trim().is_empty() {
                errors.push(ValidationError::new(
                    "browser.executable.empty",
                    "Executable must not be empty",
                    Some(path("executable")),
                ));
            }
            // Patterns checks: non-empty strings
            for (pi, p) in b.patterns.iter().enumerate() {
                if p.trim().is_empty() {
                    errors.push(ValidationError::new(
                        "pattern.empty",
                        "Pattern must not be empty",
                        Some(format!("browsers[{i}].patterns[{pi}]")),
                    ));
                }
                // Basic sanity: avoid spaces which are often mistakes
                if p.contains('\n') {
                    errors.push(ValidationError::new(
                        "pattern.newline",
                        "Pattern contains a newline",
                        Some(format!("browsers[{i}].patterns[{pi}]")),
                    ));
                }
            }
        }

        // Overlapping exact-duplicate patterns across browsers (ambiguous)
        let mut pattern_map: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, b) in self.browsers.iter().enumerate() {
            for p in &b.patterns {
                pattern_map.entry(p.as_str()).or_default().push(i);
            }
        }
        for (p, idxs) in pattern_map.iter() {
            if idxs.len() > 1 {
                let msg = format!(
                    "Pattern '{p}' appears in multiple browsers (indexes {idxs:?}); matching order applies"
                );
                errors.push(ValidationError::new(
                    "pattern.duplicate",
                    msg,
                    Some("browsers[*].patterns".to_string()),
                ));
            }
        }

        // Strict: ensure executables are resolvable from PATH
        if strict {
            for (i, b) in self.browsers.iter().enumerate() {
                if !b.executable.trim().is_empty() && which_in_path(&b.executable).is_none() {
                    errors.push(ValidationError::new(
                        "browser.executable.not_found",
                        format!("Executable '{}' not found in PATH", b.executable),
                        Some(format!("browsers[{i}].executable")),
                    ));
                }
            }
        }

        ValidationResult { errors }
    }
}

fn which_in_path(cmd: &str) -> Option<PathBuf> {
    if cmd.contains(std::path::MAIN_SEPARATOR) {
        let p = PathBuf::from(cmd);
        if p.is_file() && is_executable(&p) {
            return Some(p);
        }
        return None;
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(cmd);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(p: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .ok()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

impl ValidationResult {
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Print formatted validation errors to stderr, including a header line.
    pub fn print(&self) {
        self.print_to(io::stderr());
    }

    /// Print formatted validation errors to any writer (for tests).
    pub fn print_to<W: Write>(&self, mut w: W) {
        let count = self.errors.len();
        let _ = writeln!(w, "Found {count} validation issue(s):");
        for d in &self.errors {
            if let Some(path) = &d.path {
                let _ = writeln!(w, "- {}: {} — {}", d.code, path, d.message);
            } else {
                let _ = writeln!(w, "- {}: {}", d.code, d.message);
            }
        }
    }
}

// validate_config removed; call Config::validate(strict) directly

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_detects_empty() {
        let cfg = Config {
            browsers: vec![],
            notifications: Notifications::default(),
        };
        let res = cfg.validate(false);
        assert!(res.errors.iter().any(|e| e.code == "browsers.empty"));
    }

    #[test]
    fn validate_duplicate_names_and_empty_exec() {
        let cfg = Config {
            browsers: vec![
                Browser {
                    name: "A".into(),
                    executable: "".into(),
                    args: vec![],
                    patterns: vec![],
                },
                Browser {
                    name: "A".into(),
                    executable: "firefox".into(),
                    args: vec![],
                    patterns: vec![],
                },
            ],
            notifications: Notifications::default(),
        };
        let res = cfg.validate(false);
        assert!(
            res.errors
                .iter()
                .any(|e| e.code == "browser.name.duplicate")
        );
        assert!(
            res.errors
                .iter()
                .any(|e| e.code == "browser.executable.empty")
        );
    }

    #[test]
    fn validate_browser_name_empty() {
        let cfg = Config {
            browsers: vec![Browser {
                name: "   ".into(),
                executable: "firefox".into(),
                args: vec![],
                patterns: vec![],
            }],
            notifications: Notifications::default(),
        };
        let res = cfg.validate(false);
        assert!(res.errors.iter().any(|e| e.code == "browser.name.empty"));
    }

    #[test]
    fn validate_pattern_empty_and_newline() {
        let cfg = Config {
            browsers: vec![Browser {
                name: "B".into(),
                executable: "firefox".into(),
                args: vec![],
                patterns: vec!["".into(), "foo\nbar".into()],
            }],
            notifications: Notifications::default(),
        };
        let res = cfg.validate(false);
        assert!(res.errors.iter().any(|e| e.code == "pattern.empty"));
        assert!(res.errors.iter().any(|e| e.code == "pattern.newline"));
    }

    #[test]
    fn validate_duplicate_patterns_across_browsers() {
        let cfg = Config {
            browsers: vec![
                Browser {
                    name: "A".into(),
                    executable: "firefox".into(),
                    args: vec![],
                    patterns: vec!["example.com".into()],
                },
                Browser {
                    name: "B".into(),
                    executable: "chromium".into(),
                    args: vec![],
                    patterns: vec!["example.com".into()],
                },
            ],
            notifications: Notifications::default(),
        };
        let res = cfg.validate(false);
        assert!(res.errors.iter().any(|e| e.code == "pattern.duplicate"));
    }

    #[test]
    fn validation_result_print_format() {
        let res = ValidationResult {
            errors: vec![
                ValidationError {
                    code: "code1".into(),
                    message: "msg1".into(),
                    path: Some("path1".into()),
                },
                ValidationError {
                    code: "code2".into(),
                    message: "msg2".into(),
                    path: None,
                },
            ],
        };
        let mut buf: Vec<u8> = Vec::new();
        res.print_to(&mut buf);
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("Found 2 validation issue(s):\n"));
        assert!(s.contains("- code1: path1 — msg1\n"));
        assert!(s.contains("- code2: msg2\n"));
    }
}
