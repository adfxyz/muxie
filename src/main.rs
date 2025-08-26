mod asset;
mod browser;
mod cli;
mod client;
mod config;
mod daemon;
mod install;
mod notify;
mod open;
mod paths;
mod pattern;
mod state;
mod uninstall;

use crate::client::MuxieClient;
use crate::install::install;
use crate::open::open_url;
use crate::uninstall::uninstall;
use clap::Parser;
use cli::{Cli, Commands, ConfigCommands, DaemonCommands};

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Install {} => install(),
        Commands::Open { url: None } => {
            eprintln!("Error: No URL provided to open");
            std::process::exit(1);
        }
        Commands::Open { url: Some(url) } => {
            // Try daemon first; on any error, fall back to in-process open
            match client::ZbusClient::new().and_then(|c| c.open_url(url)) {
                Ok(()) => Ok(()),
                Err(err) => {
                    eprintln!(
                        "Daemon unavailable or failed ({err}). Falling back to direct open..."
                    );
                    open_url(url, cli.no_notify, cli.verbose)
                }
            }
        }
        Commands::Uninstall {
            yes,
            dry_run,
            restore_default,
        } => uninstall(*yes, *dry_run, *restore_default),
        Commands::Config { command } => match command {
            ConfigCommands::Validate {} => match config::read_config() {
                Ok(cfg) => {
                    let result = cfg.validate(true);
                    if result.is_empty() {
                        println!("Configuration is valid");
                        std::process::exit(0);
                    } else {
                        result.print();
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("Error validating configuration: {err}");
                    std::process::exit(2);
                }
            },
        },
        Commands::Daemon { command } => match command {
            DaemonCommands::Run {} => daemon::run(cli.no_notify, cli.verbose),
            DaemonCommands::Status {} => {
                match client::ZbusClient::is_running() {
                    Ok(true) => {
                        println!("Muxie daemon is running");
                        Ok(())
                    }
                    Ok(false) => {
                        println!("Muxie daemon is not running");
                        Ok(())
                    }
                    Err(err) => {
                        eprintln!("Unable to determine daemon status: {err}");
                        // Return an error to make exit code non-zero
                        Err(anyhow::anyhow!("status check failed"))
                    }
                }
            }
        },
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
