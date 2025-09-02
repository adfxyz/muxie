#[cfg(feature = "self-install")]
mod asset;
mod browser;
mod cli;
mod client;
mod config;
mod daemon;
mod dialog;
#[cfg(feature = "self-install")]
mod install;
mod notify;
mod open;
mod paths;
mod pattern;
#[cfg(feature = "self-install")]
mod state;
#[cfg(feature = "self-install")]
mod uninstall;
mod util;

use crate::client::MuxieClient;
#[cfg(feature = "self-install")]
use crate::install::install;
use crate::open::open_url;
#[cfg(feature = "self-install")]
use crate::uninstall::uninstall;
use clap::Parser;
use cli::{Cli, Commands, ConfigCommands, DaemonCommands};

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        #[cfg(feature = "self-install")]
        Commands::Install {} => install(),
        Commands::Open { url: None } => {
            eprintln!("Error: No URL provided to open");
            std::process::exit(1);
        }
        Commands::Open { url: Some(url) } => {
            // Try daemon first; on cancel, do not fall back. On other errors, fall back to in-process open.
            match client::ZbusClient::new().and_then(|c| c.open_url(url)) {
                Ok(()) => Ok(()),
                Err(err) => {
                    let es = err.to_string();
                    if es.contains(crate::open::CANCELED_ERR_MARKER) {
                        // Canceled by user via dialog; do not fall back.
                        eprintln!("Open canceled");
                        Err(anyhow::anyhow!("canceled"))
                    } else {
                        eprintln!(
                            "Daemon unavailable or failed ({err}). Falling back to direct open..."
                        );
                        open_url(url, cli.no_notify, cli.verbose)
                    }
                }
            }
        }
        #[cfg(feature = "self-install")]
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
            DaemonCommands::Reload {} => match client::ZbusClient::reload() {
                Ok(true) => {
                    println!("Reloaded configuration");
                    Ok(())
                }
                Ok(false) => {
                    eprintln!("Reload failed (kept previous config)");
                    Err(anyhow::anyhow!("reload failed"))
                }
                Err(err) => {
                    eprintln!("Unable to reload: {err}");
                    Err(anyhow::anyhow!("reload failed"))
                }
            },
        },
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
