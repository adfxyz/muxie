use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Browser {
    pub name: String,
    pub executable: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub patterns: Vec<String>,
}

impl Browser {
    pub fn open_url(&self, url: &str) -> Result<()> {
        let mut command = std::process::Command::new(&self.executable);
        let mut url_arg_found = false;
        for arg in &self.args {
            match arg.as_str() {
                "%u" | "%U" => {
                    url_arg_found = true;
                    command.arg(url);
                }
                _ => {
                    command.arg(arg);
                }
            }
        }
        if !url_arg_found {
            command.arg(url);
        }
        command.spawn()?;
        Ok(())
    }

    pub fn from_desktop_entry(entry: &freedesktop_desktop_entry::DesktopEntry) -> Option<Browser> {
        let (name, exec, mime_type) = match (entry.name(None), entry.exec(), entry.mime_type()) {
            (Some(name), Some(exec), Some(mime)) => (name.to_string(), exec, mime),
            _ => return None,
        };
        if !mime_type.contains("x-scheme-handler/http") {
            return None;
        }
        let executable = exec.split_whitespace().next().unwrap().to_string();
        let args = exec
            .split_whitespace()
            .skip(1)
            .map(|s| s.to_string())
            .collect();
        Some(Browser {
            name,
            executable,
            args,
            patterns: Vec::new(),
        })
    }
}
