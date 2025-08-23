mod asset;
mod browser;
mod cli;
mod config;
mod install;
mod notify;
mod paths;
mod pattern;
mod state;
mod uninstall;

use crate::install::install;
use crate::notify::{notify_error, NotifyPrefs};
use crate::uninstall::uninstall;
use anyhow::{bail, Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use config::read_config;
use pattern::Pattern;

fn open_url(url: &str, no_notify: bool) -> Result<()> {
    let config = read_config().context("Failed to load configuration")?;
    if config.browsers.is_empty() {
        bail!("No browsers configured. Run 'muxie install' to set up the browsers.");
    }
    let notify_prefs = NotifyPrefs {
        enabled: config.notifications.enabled && !no_notify,
        redact_urls: config.notifications.redact_urls,
    };
    let default_browser = &config.browsers[0];
    let mut selected_rule: Option<(String, String)> = None; // (browser_name, pattern)
    for browser in &config.browsers {
        for pattern in &browser.patterns {
            if pattern.matches(url) {
                if selected_rule.is_none() {
                    selected_rule = Some((browser.name.clone(), pattern.clone()));
                }
                match browser.open_url(url) {
                    Ok(_) => return Ok(()),
                    Err(err) => {
                        eprintln!(
                            "Warning: Failed to open URL '{}' with browser '{}': {}",
                            url, browser.name, err
                        );
                        eprintln!("Trying next browser...");
                        continue;
                    }
                }
            }
        }
    }
    let result = default_browser.open_url(url).with_context(|| {
        format!(
            "Failed to open URL '{}' with default browser '{}'",
            url, default_browser.name
        )
    });

    if let Err(err) = &result {
        // Decide rule label shown in notification
        let (rule_label, browser_label) = match &selected_rule {
            Some((bname, pat)) => (pat.as_str(), bname.as_str()),
            None => ("default", default_browser.name.as_str()),
        };
        notify_error(
            url,
            rule_label,
            browser_label,
            &format!("{err}"),
            &notify_prefs,
        );
    }
    result
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Install {} => install(),
        Commands::Open { url: None } => {
            eprintln!("Error: No URL provided to open");
            std::process::exit(1);
        }
        Commands::Open { url: Some(url) } => open_url(url, cli.no_notify),
        Commands::Uninstall {
            yes,
            dry_run,
            restore_default,
        } => uninstall(*yes, *dry_run, *restore_default),
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
