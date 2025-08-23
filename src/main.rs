mod asset;
mod browser;
mod cli;
mod config;
mod install;
mod paths;
mod pattern;
mod state;
mod uninstall;

use crate::install::install;
use crate::uninstall::uninstall;
use anyhow::{bail, Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use config::read_config;
use pattern::Pattern;

fn open_url(url: &str) -> Result<()> {
    let config = read_config().context("Failed to load configuration")?;
    if config.browsers.is_empty() {
        bail!("No browsers configured. Run 'muxie install' to set up the browsers.");
    }
    let default_browser = &config.browsers[0];
    for browser in &config.browsers {
        for pattern in &browser.patterns {
            if pattern.matches(url) {
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
    default_browser.open_url(url).with_context(|| {
        format!(
            "Failed to open URL '{}' with default browser '{}'",
            url, default_browser.name
        )
    })
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Install {} => install(),
        Commands::Open { url: None } => {
            eprintln!("Error: No URL provided to open");
            std::process::exit(1);
        }
        Commands::Open { url: Some(url) } => open_url(url),
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
