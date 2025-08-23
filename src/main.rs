mod asset;
mod browser;
mod cli;
mod config;
mod install;
mod notify;
mod open;
mod paths;
mod pattern;
mod state;
mod uninstall;

use crate::install::install;
use crate::open::open_url;
use crate::uninstall::uninstall;
use clap::Parser;
use cli::{Cli, Commands};

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
