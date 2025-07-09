use super::get_styles;
use clap::{Parser, Subcommand};

use crate::{
    cli::completion::Shell,
    commands::{
        ApplyCmd, BrewBackupCmd, BrewInstallCmd, CheckUpdateCmd, ConfigDeleteCmd, ConfigShowCmd,
        ExecCmd, FetchCmd, InitCmd, ResetCmd, SelfUpdateCmd, StatusCmd, UnapplyCmd,
        config::{lock::ConfigLockCmd, unlock::ConfigUnlockCmd},
    },
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
    /// Apply your preferences and other things.
    Apply(ApplyCmd),
    /// Run your external command(s).
    Exec(ExecCmd),
    /// Initialize a new config file.
    Init(InitCmd),
    /// Unapply the previously applied modifications(s).
    Unapply(UnapplyCmd),
    /// (DANGEROUS) Hard-reset all preferences.
    Reset(ResetCmd),
    /// Compare your system against your config.
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
    /// Updates cutler itself (for manual installs).
    SelfUpdate(SelfUpdateCmd),
    /// Generate shell completions.
    Completion {
        /// Your shell type.
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Sync the local config with remote defined in [remote].
    Fetch(FetchCmd),
}

#[derive(Subcommand, Debug)]
pub enum BrewSubcmd {
    /// Backup current formulae and casks into config.
    Backup(BrewBackupCmd),
    /// Install Homebrew if not present, then install all formulae and casks from config.
    Install(BrewInstallCmd),
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcmd {
    /// Display the contents of the config.
    Show(ConfigShowCmd),
    /// Delete the config file.
    Delete(ConfigDeleteCmd),
    /// Lock the config file.
    Lock(ConfigLockCmd),
    /// Lock the config file.
    Unlock(ConfigUnlockCmd),
}
