use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Disable desktop notifications for this run
    #[arg(long = "no-notify")]
    pub no_notify: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Make muxie the default browser. This is required to work properly.
    Install {},

    /// Open URL
    Open { url: Option<String> },

    /// Uninstall muxie assets and optionally restore previous default browser
    Uninstall {
        /// Confirm all prompts (uninstall and delete config)
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Show what would be done without making changes
        #[arg(long = "dry-run")]
        dry_run: bool,

        /// Attempt to restore the previous default browser if a backup exists
        #[arg(long = "restore-default")]
        restore_default: bool,
    },

    /// Config-related commands
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Validate the configuration file (strict mode)
    Validate {},
}
