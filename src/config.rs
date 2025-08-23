use crate::browser::Browser;
use crate::paths::config_path;
use anyhow::{bail, Context, Result};
use freedesktop_desktop_entry::{default_paths, Iter};
use serde::{Deserialize, Serialize};
 

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub browsers: Vec<Browser>,
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
        };
        let config_text = serde_yaml::to_string(&config)
            .context("Failed to serialize default config")?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        std::fs::write(&config_path, config_text)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
    }
    Ok(())
}
