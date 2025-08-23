use crate::paths::{config_path, desktop_entry_path, icon_paths, state_path};
use crate::state::{read_state, remove_state_file};
use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

fn confirm(prompt: &str, auto_yes: bool) -> Result<bool> {
    if auto_yes {
        return Ok(true);
    }
    print!("{prompt} ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let ans = input.trim().to_lowercase();
    Ok(matches!(ans.as_str(), "y" | "yes"))
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
                    "Warning: xdg-settings {:?} exited with code {:?}\nstdout: {}\nstderr: {}\nHints:\n  - Ensure xdg-utils is installed and your desktop environment is supported.\n  - Try: 'xdg-settings get default-web-browser' and 'xdg-settings check default-web-browser'.\n  - You can set the default browser manually via your system settings.",
                    args,
                    output.status.code(),
                    if stdout.is_empty() { "<empty>" } else { &stdout },
                    if stderr.is_empty() { "<empty>" } else { &stderr },
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

fn restore_previous_default_browser_from_backup() -> Result<()> {
    if let Some(state) = read_state()? {
        if let Some(prev) = state.previous_default_browser.as_deref() {
            run_xdg_settings_with_diagnostics(&["set", "default-web-browser", prev]);
            run_xdg_settings_with_diagnostics(&["set", "default-url-scheme-handler", "http", prev]);
            run_xdg_settings_with_diagnostics(&[
                "set",
                "default-url-scheme-handler",
                "https",
                prev,
            ]);
            run_xdg_settings_with_diagnostics(&["set", "default-url-scheme-handler", "ftp", prev]);
        }
    }
    Ok(())
}

pub fn uninstall(yes: bool, dry_run: bool, restore_default: bool) -> Result<()> {
    // Confirm uninstall
    if !confirm(
        "This will remove Muxie desktop entry and icons. Proceed? [y/N]",
        yes,
    )? {
        println!("Aborted.");
        return Ok(());
    }

    let desktop = desktop_entry_path();
    let icons = icon_paths();
    let cfg = config_path();
    let state = state_path();

    println!("Planned actions:");
    println!("- Remove desktop entry: {}", desktop.display());
    for p in &icons {
        println!("- Remove icon: {}", p.display());
    }
    println!("- Remove state file: {}", state.display());

    let mut delete_config = false;
    if yes {
        delete_config = true; // yes confirms all prompts
    } else if confirm(&format!("Delete {} as well? [y/N]", cfg.display()), false)? {
        delete_config = true;
    }
    if delete_config {
        println!("- Remove config: {}", cfg.display());
    }

    if dry_run {
        println!("Dry run: no changes were made.");
        return Ok(());
    }

    // Attempt restore first if requested
    if restore_default {
        if let Err(e) = restore_previous_default_browser_from_backup() {
            eprintln!("Warning: failed to restore previous default browser: {e}");
        }
    }

    let mut failures: Vec<(PathBuf, String)> = Vec::new();
    let mut removed: Vec<PathBuf> = Vec::new();

    // Remove desktop entry
    if desktop.exists() {
        match fs::remove_file(&desktop) {
            Ok(_) => removed.push(desktop.clone()),
            Err(e) => failures.push((desktop.clone(), e.to_string())),
        }
    }
    // Remove icons
    for p in &icons {
        if p.exists() {
            if let Err(e) = fs::remove_file(p) {
                failures.push((p.clone(), e.to_string()));
            } else {
                removed.push(p.clone());
            }
        }
    }
    // Remove state file (after restore attempt)
    if state.exists() {
        if let Err(e) = remove_state_file() {
            failures.push((state.clone(), e.to_string()));
        } else {
            removed.push(state.clone());
        }
    }
    // Remove config if approved
    if delete_config && cfg.exists() {
        if let Err(e) = fs::remove_file(&cfg) {
            failures.push((cfg.clone(), e.to_string()));
        } else {
            removed.push(cfg.clone());
        }
    }

    println!("\nSummary:");
    for p in &removed {
        println!("- Removed: {}", p.display());
    }
    if !failures.is_empty() {
        eprintln!("- Failed to remove {} item(s):", failures.len());
        for (p, e) in &failures {
            eprintln!("  {} -> {}", p.display(), e);
        }
        anyhow::bail!("uninstall completed with failures");
    }
    Ok(())
}
