use super::get_styles;
use clap::{Parser, Subcommand, ValueEnum};

use crate::commands::{
    ApplyCmd, BrewBackupCmd, BrewInstallCmd, CheckUpdateCmd, ConfigDeleteCmd, ConfigShowCmd,
    ExecCmd, InitCmd, ResetCmd, SelfUpdateCmd, StatusCmd, UnapplyCmd,
};

#[derive(Parser)]
#[command(name = "cutler", styles = get_styles(), version, about)]
pub struct Args {
    /// Increase output verbosity.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors and warnings.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Do not restart system services after command execution.
    #[arg(short, long, global = true)]
    pub no_restart_services: bool,

    /// Run in dry-run mode. Commands will be printed but not executed.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Accepts all interactive prompts.
    #[arg(short = 'y', long, global = true)]
    pub accept_interactive: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Apply the changes written in your config file.
    Apply(ApplyCmd),
    /// Run only the external commands written in the config file.
    Exec(ExecCmd),
    /// Initialize a new config file with sensible defaults.
    Init(InitCmd),
    /// Unapply the previously applied modifications(s).
    Unapply(UnapplyCmd),
    /// Hard reset domains written in the config file (dangerous).
    Reset(ResetCmd),
    /// Display current status comparing the config and the system.
    Status(StatusCmd),
    /// Homebrew backup-and-restore related commands.
    Brew {
        #[command(subcommand)]
        command: BrewSubcmd,
    },
    /// Manage the configuration file.
    Config {
        #[command(subcommand)]
        command: ConfigSubcmd,
    },
    /// Check for version updates.
    CheckUpdate(CheckUpdateCmd),
    /// Updates cutler itself (only for manual installs).
    SelfUpdate(SelfUpdateCmd),
    /// Generate shell completions.
    Completion {
        /// Shell type to generate completions for.
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum BrewSubcmd {
    /// Backup current formulae and casks into config file.
    Backup(BrewBackupCmd),
    /// Install Homebrew if not present, then install all formulae and casks from config.
    Install(BrewInstallCmd),
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcmd {
    /// Display the contents of the configuration file.
    Show(ConfigShowCmd),
    /// Delete the configuration file.
    Delete(ConfigDeleteCmd),
}

// Shell enum and other value enums remain unchanged
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    PowerShell,
}
