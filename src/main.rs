mod asset;
mod browser;
mod cli;
mod config;
mod install;
mod pattern;

use crate::install::install;
use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, Commands};
use config::read_config;
use pattern::Pattern;

fn open_url(url: &str) -> Result<()> {
    let config = read_config();
    if config.browsers.is_empty() {
        bail!("No browsers configured");
    }
    let default_browser = &config.browsers[0];
    for browser in &config.browsers {
        for pattern in &browser.patterns {
            if pattern.matches(url) {
                match browser.open_url(url) {
                    Ok(_) => return Ok(()),
                    Err(_) => continue,
                }
            }
        }
    }
    default_browser.open_url(url)?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install {} => {
            install().expect("Unable to install");
        }
        Commands::Open { url: None } => {
            println!("open called without URL")
        }
        Commands::Open { url: Some(url) } => {
            open_url(url).expect("Unable to open URL");
        }
    }
}
