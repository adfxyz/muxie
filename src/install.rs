use crate::asset::{Asset, Icon};
use crate::config::ensure_config;
use anyhow::{Context, Result};
use std::path::PathBuf;

const DESKTOP_ENTRY_NAME: &str = "muxie.desktop";

pub fn install() -> Result<()> {
    ensure_config().context("Failed to create default configuration")?;
    install_icons().context("Failed to install icons")?;
    let desktop_entry_path = create_desktop_entry().context("Failed to create desktop entry")?;
    make_default_browser(desktop_entry_path).context("Failed to set as default browser")?;
    Ok(())
}

fn install_icons() -> Result<()> {
    for icon in Icon::iter() {
        let icon_embed = Icon::get(icon.as_ref())
            .with_context(|| format!("Failed to get embedded icon: {}", icon))?;
        let (size, name) = icon.split_once('/')
            .with_context(|| format!("Invalid icon path format: {}", icon))?;
        let icon_path = icon_path(size, name);

        // Ensure the parent directory exists
        if let Some(parent) = icon_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create icon directory: {}", parent.display()))?;
        }

        std::fs::write(&icon_path, icon_embed.data)
            .with_context(|| format!("Failed to write icon file: {}", icon_path.display()))?;
    }
    Ok(())
}

fn create_desktop_entry() -> Result<PathBuf> {
    let desktop_entry_path = desktop_entry_path();

    // Ensure the parent directory exists
    if let Some(parent) = desktop_entry_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create applications directory: {}", parent.display()))?;
    }

    let desktop_entry_content = Asset::get(DESKTOP_ENTRY_NAME)
        .with_context(|| format!("Failed to get embedded desktop entry: {}", DESKTOP_ENTRY_NAME))?
        .data;

    std::fs::write(&desktop_entry_path, desktop_entry_content)
        .with_context(|| format!("Failed to write desktop entry: {}", desktop_entry_path.display()))?;
    Ok(desktop_entry_path)
}

fn make_default_browser(desktop_entry_path: PathBuf) -> Result<()> {
    let file_name = desktop_entry_path.file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("Invalid desktop entry path: {}", desktop_entry_path.display()))?;
    
    for args in &[
        vec!["set", "default-web-browser", file_name],
        vec!["set", "default-url-scheme-handler", "http", file_name],
        vec!["set", "default-url-scheme-handler", "https", file_name],
        vec!["set", "default-url-scheme-handler", "ftp", file_name],
    ] {
        std::process::Command::new("xdg-settings")
            .args(args)
            .spawn()
            .with_context(|| format!("Failed to run xdg-settings with args: {:?}", args))?;
    }
    Ok(())
}

fn desktop_entry_path() -> PathBuf {
    let mut path = dirs::data_dir().expect("Failed to get user data directory");
    path.push("applications");
    path.push(DESKTOP_ENTRY_NAME);
    path
}

fn icon_path(size: &str, name: &str) -> PathBuf {
    let mut path = dirs::data_dir().expect("Failed to get user data directory");
    path.push("icons");
    path.push("hicolor");
    path.push(size);
    path.push("apps");
    path.push(name);
    path
}
