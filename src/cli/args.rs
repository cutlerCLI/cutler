use clap::{Parser, Subcommand, ValueEnum};

/// top‐level CLI args for cutler
#[derive(Parser)]
#[command(name = "cutler", version, about)]
pub struct Args {
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
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Apply defaults, and execute the external commands from the config file.
    Apply,
    /// Run only the external commands assigned in the config file.
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
        /// Skip confirmation prompt.
        #[arg(short, long)]
        force: bool,
    },
    /// Display current status comparing the config and the system.
    Status,
    /// Manage the configuration file.
    Config {
        #[command(subcommand)]
        command: ConfigSub,
    },
    /// Generate shell completions.
    Completion {
        /// Shell type to generate completions for.
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Check for version updates.
    CheckUpdate,
}

#[derive(Subcommand)]
pub enum ConfigSub {
    /// Display the contents of the configuration file.
    Show,
    /// Delete the configuration file.
    Delete,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    PowerShell,
}
