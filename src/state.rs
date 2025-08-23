use crate::paths::state_path;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InstallState {
    /// e.g., "firefox.desktop"
    pub previous_default_browser: Option<String>,
}

pub fn read_state() -> Result<Option<InstallState>> {
    let path = state_path();
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read state file: {}", path.display()))?;
    let state: InstallState = serde_yaml::from_str(&text)
        .with_context(|| format!("Failed to parse state file: {}", path.display()))?;
    Ok(Some(state))
}

pub fn write_state(state: &InstallState) -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let text = serde_yaml::to_string(state)?;
    fs::write(&path, text)
        .with_context(|| format!("Failed to write state file: {}", path.display()))?;
    Ok(())
}

pub fn remove_state_file() -> Result<()> {
    let path = state_path();
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to remove state file: {}", path.display()))?;
    }
    Ok(())
}
