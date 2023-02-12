use crate::asset::{Asset, Icon};
use crate::config::ensure_config;
use std::io::Result;
use std::path::PathBuf;

const DESKTOP_ENTRY_NAME: &str = "browser-demux.desktop";

pub fn install() -> Result<()> {
    ensure_config();
    install_icons()?;
    let desktop_entry_path = create_desktop_entry()?;
    make_default_browser(desktop_entry_path)?;
    Ok(())
}

fn install_icons() -> Result<()> {
    for icon in Icon::iter() {
        let icon_embed = Icon::get(icon.as_ref()).unwrap();
        let (size, name) = icon.split_once('/').unwrap();
        let icon_path = icon_path(size, name);
        std::fs::write(icon_path, icon_embed.data)?;
    }
    Ok(())
}

fn create_desktop_entry() -> Result<PathBuf> {
    let desktop_entry_path = desktop_entry_path();
    std::fs::write(
        &desktop_entry_path,
        Asset::get(DESKTOP_ENTRY_NAME).unwrap().data,
    )?;
    Ok(desktop_entry_path)
}

fn make_default_browser(desktop_entry_path: PathBuf) -> Result<()> {
    let file_name = desktop_entry_path.file_name().unwrap().to_str().unwrap();
    for args in &[
        vec!["set", "default-web-browser", file_name],
        vec!["set", "default-url-scheme-handler", "http", file_name],
        vec!["set", "default-url-scheme-handler", "https", file_name],
        vec!["set", "default-url-scheme-handler", "ftp", file_name],
    ] {
        let mut command = std::process::Command::new("xdg-settings");
        command.args(args);
        command.spawn()?;
    }
    Ok(())
}

fn desktop_entry_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap();
    path.push("applications");
    path.push(DESKTOP_ENTRY_NAME);
    path
}

fn icon_path(size: &str, name: &str) -> PathBuf {
    let mut path = dirs::data_dir().unwrap();
    path.push("icons");
    path.push("hicolor");
    path.push(size);
    path.push("apps");
    path.push(name);
    path
}
