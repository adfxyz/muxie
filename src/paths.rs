use crate::asset::Icon;
use std::path::PathBuf;

pub fn desktop_entry_path() -> PathBuf {
    let mut path = dirs::data_dir().expect("Failed to get user data directory");
    path.push("applications");
    path.push("muxie.desktop");
    path
}

pub fn icon_path(size: &str, name: &str) -> PathBuf {
    let mut path = dirs::data_dir().expect("Failed to get user data directory");
    path.push("icons");
    path.push("hicolor");
    path.push(size);
    path.push("apps");
    path.push(name);
    path
}

pub fn icon_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for icon in Icon::iter() {
        if let Some((size, name)) = icon.as_ref().split_once('/') {
            out.push(icon_path(size, name));
        }
    }
    out
}

pub fn config_path() -> PathBuf {
    let mut config_dir = dirs::config_dir().expect("Failed to get user config directory");
    config_dir.push("muxie.toml");
    config_dir
}

pub fn state_path() -> PathBuf {
    let mut p = dirs::state_dir().expect("Failed to get user state directory");
    p.push("muxie");
    p.push("state.toml");
    p
}
