use crate::asset::{Asset, Icon};
use crate::config::ensure_config;
use crate::paths::{dbus_service_dir, dbus_service_path, desktop_entry_path, icon_path};
use crate::state::{InstallState, write_state};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn install() -> Result<()> {
    ensure_config().context("Failed to create default configuration")?;
    install_icons().context("Failed to install icons")?;
    let desktop_entry_path = create_desktop_entry().context("Failed to create desktop entry")?;
    create_dbus_service().context("Failed to install D-Bus activation service")?;
    // Best-effort backup of previous default browser before we change it
    backup_previous_default_browser().ok();
    make_default_browser(desktop_entry_path).context("Failed to set as default browser")?;
    Ok(())
}

fn backup_previous_default_browser() -> Result<()> {
    // Query previous default and persist for potential restoration.
    let output = std::process::Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output();
    let mut state = InstallState::default();
    if let Ok(out) = output
        && out.status.success()
    {
        let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !val.is_empty() {
            state.previous_default_browser = Some(val);
        }
    }
    // Don't fail installation if backup fails; best-effort.
    let _ = write_state(&state);
    Ok(())
}

fn install_icons() -> Result<()> {
    for icon in Icon::iter() {
        let icon_embed = Icon::get(icon.as_ref())
            .with_context(|| format!("Failed to get embedded icon: {icon}"))?;
        let (size, name) = icon
            .split_once('/')
            .with_context(|| format!("Invalid icon path format: {icon}"))?;
        let icon_path = icon_path(size, name);

        // Ensure the parent directory exists
        if let Some(parent) = icon_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create icon directory: {}", parent.display())
            })?;
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
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create applications directory: {}",
                parent.display()
            )
        })?;
    }

    let desktop_entry_content = Asset::get("muxie.desktop")
        .with_context(|| format!("Failed to get embedded desktop entry: {}", "muxie.desktop"))?
        .data;

    std::fs::write(&desktop_entry_path, desktop_entry_content).with_context(|| {
        format!(
            "Failed to write desktop entry: {}",
            desktop_entry_path.display()
        )
    })?;
    Ok(desktop_entry_path)
}

fn create_dbus_service() -> Result<PathBuf> {
    let service_path = dbus_service_path();
    if let Some(dir) = service_path.parent() {
        std::fs::create_dir_all(dir).with_context(|| {
            format!(
                "Failed to create D-Bus service directory: {}",
                dir.display()
            )
        })?;
    }
    // Use the plain command name so activation relies on PATH resolution.
    // This avoids baking an absolute path that may change across installs.
    let content = format!(
        "[D-BUS Service]\nName={}\nExec=muxie daemon run\n",
        crate::daemon::DBUS_SERVICE,
    );
    std::fs::write(&service_path, content).with_context(|| {
        format!(
            "Failed to write D-Bus service file: {}",
            service_path.display()
        )
    })?;

    // Ensure directory listing exists for parent chain (no-op if present)
    let _ = dbus_service_dir();
    Ok(service_path)
}

fn make_default_browser(desktop_entry_path: PathBuf) -> Result<()> {
    let file_name = desktop_entry_path
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| {
            format!(
                "Invalid desktop entry path: {}",
                desktop_entry_path.display()
            )
        })?;

    // Issue four separate handler assignments; on failure, print detailed diagnostics
    run_xdg_settings_with_diagnostics(&["set", "default-web-browser", file_name]);
    run_xdg_settings_with_diagnostics(&["set", "default-url-scheme-handler", "http", file_name]);
    run_xdg_settings_with_diagnostics(&["set", "default-url-scheme-handler", "https", file_name]);
    run_xdg_settings_with_diagnostics(&["set", "default-url-scheme-handler", "ftp", file_name]);
    Ok(())
}

fn run_xdg_settings_with_diagnostics(args: &[&str]) {
    match std::process::Command::new("xdg-settings")
        .args(args)
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                eprintln!(
                    "Warning: xdg-settings {:?} exited with code {:?}\nstdout: {}\nstderr: {}\nHints:\n  - Ensure xdg-utils is installed and your desktop environment is supported.\n  - Try: 'xdg-settings get default-web-browser' and 'xdg-settings check default-web-browser'.\n  - You can set the default browser manually via your system settings or with: xdg-settings set default-web-browser muxie.desktop",
                    args,
                    output.status.code().unwrap(),
                    if stdout.is_empty() {
                        "<empty>"
                    } else {
                        &stdout
                    },
                    if stderr.is_empty() {
                        "<empty>"
                    } else {
                        &stderr
                    },
                );
            }
        }
        Err(err) => {
            eprintln!(
                "Warning: failed to invoke xdg-settings {args:?}: {err}\nHints:\n  - Ensure 'xdg-settings' (xdg-utils) is installed and in PATH."
            );
        }
    }
}
