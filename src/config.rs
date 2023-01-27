use crate::browser::Browser;
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
        browsers: vec![Browser {
            name: "firefox".to_string(),
            executable: "firefox".to_string(),
            args: Vec::new(),
            patterns: Vec::new(),
        }],
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
