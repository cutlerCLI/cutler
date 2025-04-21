use clap::{Parser, Subcommand, ValueEnum};

/// Declarative macOS settings management at your fingertips, with speed.
#[derive(Parser)]
#[command(name = "cutler", version, about)]
pub struct Cli {
    /// Increase output verbosity.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Do not restart system services after command execution.
    #[arg(short, long, global = true)]
    pub no_restart_services: bool,

    /// Run in dry-run mode. Commands will be printed but not executed.
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Apply defaults from the config file.
    Apply,
    /// Initialize a new configuration file with sensible defaults.
    Init {
        #[arg(short, long)]
        force: bool,
    },
    /// Unapply (delete) defaults from the config file.
    Unapply,
    /// Hard reset domains in the config file (dangerous, ignores snapshots)
    Reset {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Display current status comparing the config vs current defaults.
    Status,
    /// Manage the configuration file.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Generate shell completions
    Completion {
        /// Shell type to generate completions for (bash or zsh)
        #[arg(value_enum)]
        shell: Shell,

        /// Directory where to write the completion script
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
    /// Check for version updates
    CheckUpdate
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Display the contents of the configuration file.
    Show,
    /// Delete the configuration file.
    Delete,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    /// Generate completions for bash
    Bash,
    /// Generate completions for zsh
    Zsh,
}
