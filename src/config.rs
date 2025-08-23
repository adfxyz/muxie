use crate::browser::Browser;
use crate::paths::config_path;
use anyhow::{Context, Result, bail};
use freedesktop_desktop_entry::{Iter, default_paths};
use serde::{Deserialize, Serialize};

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
