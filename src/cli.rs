use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Disable desktop notifications for this run
    #[arg(long = "no-notify")]
    pub no_notify: bool,

    /// Verbose output (use multiple times for increased verbosity)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Make muxie the default browser. This is required to work properly.
    Install {},

    /// Open URL
    Open { url: Option<String> },

    /// Daemon-related commands
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },

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

#[derive(Subcommand)]
pub enum DaemonCommands {
    /// Run the daemon in the foreground (manual start)
    Run {},

    /// Show whether the daemon is currently running
    Status {},
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_open_with_url() {
        let cli = Cli::parse_from(["muxie", "open", "https://example.com"]);
        assert!(!cli.no_notify);
        assert_eq!(cli.verbose, 0);
        match cli.command {
            Commands::Open { url } => assert_eq!(url.as_deref(), Some("https://example.com")),
            _ => panic!("expected Open command"),
        }
    }

    #[test]
    fn parse_daemon_run() {
        let cli = Cli::parse_from(["muxie", "daemon", "run"]);
        match cli.command {
            Commands::Daemon { command } => match command {
                DaemonCommands::Run {} => {}
                _ => panic!("unexpected daemon subcommand"),
            },
            _ => panic!("expected Daemon Run command"),
        }
    }

    #[test]
    fn parse_daemon_status() {
        let cli = Cli::parse_from(["muxie", "daemon", "status"]);
        match cli.command {
            Commands::Daemon { command } => match command {
                DaemonCommands::Status {} => {}
                _ => panic!("unexpected daemon subcommand"),
            },
            _ => panic!("expected Daemon Status command"),
        }
    }

    #[test]
    fn parse_global_flags() {
        let cli = Cli::parse_from(["muxie", "--no-notify", "-vv", "open", "https://x"]);
        assert!(cli.no_notify);
        assert_eq!(cli.verbose, 2);
        match cli.command {
            Commands::Open { url } => assert_eq!(url.as_deref(), Some("https://x")),
            _ => panic!("expected Open command"),
        }
    }
}
