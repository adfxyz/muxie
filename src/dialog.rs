use anyhow::{Context, Result};

use crate::util::which_in_path;
use std::process::Command;

/// A simple selector interface for choosing among options via a graphical dialog.
/// Implementations should return:
/// - Ok(Some(index)) when the user selects an option
/// - Ok(None) when the user cancels/closes the dialog
/// - Err(_) for provider/runtime errors (e.g., backend invocation failed)
pub(crate) trait Selector {
    fn choose(
        &self,
        title: &str,
        message: &str,
        options: &[String],
        default_idx: usize,
    ) -> Result<Option<usize>>;
}

/// Auto-detecting selector that invokes a native dialog if possible.
///
/// Behavior:
/// - Detects a GUI session (WAYLAND_DISPLAY/DISPLAY) and prefers providers in order:
///   kdialog → zenity → yad.
/// - On successful selection, returns Ok(Some(index)).
/// - On user cancel/close, returns Ok(None).
/// - If no GUI session or no provider is available, returns Err(..) so the caller
///   can skip prompting and continue non-interactively.
#[derive(Default, Clone, Copy)]
pub(crate) struct AutoSelector;

impl AutoSelector {
    pub(crate) fn new() -> Self {
        AutoSelector
    }
}

impl Selector for AutoSelector {
    fn choose(
        &self,
        _title: &str,
        _message: &str,
        _options: &[String],
        _default_idx: usize,
    ) -> Result<Option<usize>> {
        // Detect an available provider and delegate. If no GUI session or no
        // provider is available, return an error to signal the caller to skip
        // prompting and proceed with default non-interactive behavior.
        if !have_gui_env() {
            return Err(anyhow::anyhow!("no GUI session"));
        }
        match detect_provider() {
            Some(Provider::Kdialog) => {
                KdialogSelector.choose(_title, _message, _options, _default_idx)
            }
            Some(Provider::Zenity) => {
                ZenitySelector.choose(_title, _message, _options, _default_idx)
            }
            Some(Provider::Yad) => YadSelector.choose(_title, _message, _options, _default_idx),
            None => Err(anyhow::anyhow!("no dialog provider available")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    Kdialog,
    Zenity,
    Yad,
}

fn have_gui_env() -> bool {
    std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn detect_provider() -> Option<Provider> {
    // Order: kdialog -> zenity -> yad
    if which_in_path("kdialog").is_some() {
        return Some(Provider::Kdialog);
    }
    if which_in_path("zenity").is_some() {
        return Some(Provider::Zenity);
    }
    if which_in_path("yad").is_some() {
        return Some(Provider::Yad);
    }
    None
}

struct KdialogSelector;
struct ZenitySelector;
struct YadSelector;

impl Selector for KdialogSelector {
    fn choose(
        &self,
        title: &str,
        message: &str,
        options: &[String],
        default_idx: usize,
    ) -> Result<Option<usize>> {
        if options.is_empty() {
            return Ok(None);
        }
        let mut cmd = Command::new("kdialog");
        // Using radiolist for a single-choice selection. We attach stable tags (indices)
        // so we can map back to the option index regardless of label text.
        cmd.arg("--title").arg(title);
        cmd.arg("--radiolist").arg(message);
        for (i, label) in options.iter().enumerate() {
            let tag = format!("{}", i);
            let state = if i == default_idx { "on" } else { "off" };
            cmd.arg(&tag).arg(label).arg(state);
        }
        let output = cmd.output().context("failed to run kdialog")?;
        if !output.status.success() {
            // kdialog returns non-zero on cancel or error. Treat as cancel (no selection).
            return Ok(None);
        }
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            return Ok(None);
        }
        // stdout should contain the selected tag (index)
        match stdout.parse::<usize>() {
            Ok(idx) if idx < options.len() => Ok(Some(idx)),
            _ => Ok(None),
        }
    }
}

impl Selector for ZenitySelector {
    fn choose(
        &self,
        title: &str,
        message: &str,
        options: &[String],
        _default_idx: usize,
    ) -> Result<Option<usize>> {
        if options.is_empty() {
            return Ok(None);
        }
        let mut cmd = Command::new("zenity");
        cmd.arg("--list")
            .arg("--title")
            .arg(title)
            .arg("--text")
            .arg(message)
            .arg("--column")
            .arg("Browser")
            .arg("--hide-header");
        for label in options.iter() {
            cmd.arg(label);
            // zenity's --list does not support preselecting a default row; default_idx is ignored here
        }
        let output = cmd.output().context("failed to run zenity")?;
        if !output.status.success() {
            return Ok(None);
        }
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            return Ok(None);
        }
        // Map selected label back to index
        if let Some(idx) = options.iter().position(|s| s == &stdout) {
            Ok(Some(idx))
        } else {
            Ok(None)
        }
    }
}

impl Selector for YadSelector {
    fn choose(
        &self,
        title: &str,
        message: &str,
        options: &[String],
        _default_idx: usize,
    ) -> Result<Option<usize>> {
        if options.is_empty() {
            return Ok(None);
        }
        let mut cmd = Command::new("yad");
        cmd.arg("--list")
            .arg("--title")
            .arg(title)
            .arg("--text")
            .arg(message)
            .arg("--column")
            .arg("Browser")
            .arg("--hide-header")
            // Ensure stdout contains exactly the first column value
            .arg("--print-column=1")
            // Be explicit about buttons and their exit codes
            .arg("--button=OK:0")
            .arg("--button=Cancel:1");
        for label in options.iter() {
            cmd.arg(label);
            // yad's --list does not support preselecting a default row; default_idx is ignored here
        }
        let output = cmd.output().context("failed to run yad")?;
        if !output.status.success() {
            return Ok(None);
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first = stdout.lines().next().unwrap_or("").trim();
        if first.is_empty() {
            return Ok(None);
        }
        // Yad may append the column separator (default '|') even for single column output.
        // Be robust: split at '|' and take the first segment.
        let selected = first.split('|').next().unwrap_or("").trim();
        if selected.is_empty() {
            return Ok(None);
        }
        // Map selected label back to index
        if let Some(idx) = options.iter().position(|s| s == selected) {
            Ok(Some(idx))
        } else {
            Ok(None)
        }
    }
}
