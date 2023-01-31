use crate::config::ensure_config;
use freedesktop_desktop_entry::{default_paths, Iter};
use std::ffi::OsStr;
use std::io::Result;
use std::path::PathBuf;

const DESKTOP_ENTRY: &str = r#"#!/usr/bin/env xdg-open
[Desktop Entry]
Version=1.0
Name=Browser Demux RS
Keywords=Internet;WWW;Browser;Web
Exec=browser-demux open %u
Terminal=false
X-MultipleArgs=false
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/ftp;
StartupNotify=false
Icon=browser-demux
"#;

const DESKTOP_ENTRY_NAME: &str = "browser-demux.desktop";

pub fn install() -> Result<()> {
    ensure_config();
    let desktop_entry_path = create_desktop_entry()?;
    make_default_browser(desktop_entry_path)?;
    Ok(())
}

pub fn create_desktop_entry() -> Result<PathBuf> {
    for path in Iter::new(default_paths()) {
        if path.file_name() == Some(OsStr::new(DESKTOP_ENTRY_NAME)) {
            return Ok(path);
        }
    }
    let desktop_entry_path = desktop_entry_path();
    std::fs::write(&desktop_entry_path, DESKTOP_ENTRY)?;
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
