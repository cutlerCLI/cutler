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
    /// Apply defaults, and execute the external commands from the config file.
    Apply,
    /// Run the external commands assigned in the config file.
    Cmd,
    /// Initialize a new configuration file with sensible defaults.
    Init {
        #[arg(short, long)]
        force: bool,
    },
    /// Unapply the previously applied modifications(s).
    Unapply,
    /// Hard reset domains written in the config file (dangerous).
    Reset {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Display current status comparing the config and the system.
    Status,
    /// Manage the configuration file.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Generate shell completions.
    Completion {
        /// Shell type to generate completions for (bash or zsh).
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Check for version updates.
    CheckUpdate,
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
    /// Generate completions for fish
    Fish,
    /// Generate completions for elvish
    Elvish,
    /// Generate completions for powershell
    PowerShell,
}
