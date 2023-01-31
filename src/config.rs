use crate::browser::Browser;
use freedesktop_desktop_entry::{default_paths, Iter};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILE: &str = "browser-demux.yml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub browsers: Vec<Browser>,
}

fn config_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap();
    config_dir.join(CONFIG_FILE)
}

fn default_config() -> Config {
    Config {
        browsers: installed_browsers(),
    }
}

pub fn read_config() -> Config {
    let config_path = config_path();
    if !config_path.exists() {
        return default_config();
    }
    let config_text = std::fs::read_to_string(config_path).expect("Unable to read config file");
    let config: Config = serde_yaml::from_str(&config_text).expect("Unable to parse config file");
    config
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
                    if browser.name.contains("Browser Demux") {
                        return None;
                    }
                    Some(browser)
                }
                None => None,
            }
        })
        .collect()
}

pub fn ensure_config() {
    let config_path = config_path();
    if !config_path.exists() {
        let config = default_config();
        let config_text = serde_yaml::to_string(&config).expect("Unable to serialize config");
        std::fs::write(config_path, config_text).expect("Unable to write config file");
    }
}
