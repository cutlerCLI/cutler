use super::get_styles;
use clap::{Parser, Subcommand};

use crate::commands::{
    ApplyCmd, BrewBackupCmd, BrewInstallCmd, CheckUpdateCmd, CompletionCmd, ConfigDeleteCmd,
    ConfigShowCmd, ExecCmd, FetchCmd, InitCmd, ResetCmd, SelfUpdateCmd, StatusCmd, UnapplyCmd,
    config::{lock::ConfigLockCmd, unlock::ConfigUnlockCmd},
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

    /// Do not sync with remote config (if autosync = true).
    #[arg(long, global = true)]
    pub no_sync: bool,

    /// Run in dry-run mode. Commands will be printed but not executed.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Accepts all interactive prompts.
    #[arg(short = 'y', long, global = true)]
    pub accept_all: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Apply your preferences and other things.
    #[command(visible_alias = "a")]
    Apply(ApplyCmd),
    /// Run your external command(s).
    #[command(visible_alias = "x")]
    Exec(ExecCmd),
    /// Initialize a new config file.
    #[command(visible_alias = "i")]
    Init(InitCmd),
    /// Unapply the previously applied modifications(s).
    #[command(visible_alias = "u")]
    Unapply(UnapplyCmd),
    /// (DANGEROUS) Hard-reset all preferences.
    #[command(visible_alias = "r")]
    Reset(ResetCmd),
    /// Compare your system against your config.
    #[command(visible_alias = "s")]
    Status(StatusCmd),
    /// Homebrew backup-and-restore related commands.
    #[command(visible_alias = "b")]
    Brew {
        #[command(subcommand)]
        command: BrewSubcmd,
    },
    /// Manage the configuration file.
    #[command(visible_alias = "c")]
    Config {
        #[command(subcommand)]
        command: ConfigSubcmd,
    },
    /// Check for version updates.
    #[command(visible_alias = "cu")]
    CheckUpdate(CheckUpdateCmd),
    /// Updates cutler itself (for manual installs).
    #[command(visible_alias = "up")]
    SelfUpdate(SelfUpdateCmd),
    /// Generate shell completions.
    #[command(visible_alias = "comp")]
    Completion(CompletionCmd),
    /// Sync the local config with remote defined in [remote].
    #[command(visible_alias = "f")]
    Fetch(FetchCmd),
}

#[derive(Subcommand, Debug)]
pub enum BrewSubcmd {
    /// Backup current formulae and casks into config.
    #[command(visible_alias = "b")]
    Backup(BrewBackupCmd),
    /// Install Homebrew if not present, then install all formulae and casks from config.
    #[command(visible_alias = "i")]
    Install(BrewInstallCmd),
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcmd {
    /// Display the contents of the config.
    #[command(visible_alias = "s")]
    Show(ConfigShowCmd),
    /// Delete the config file.
    #[command(visible_alias = "d")]
    Delete(ConfigDeleteCmd),
    /// Lock the config file.
    #[command(visible_alias = "l")]
    Lock(ConfigLockCmd),
    /// Lock the config file.
    #[command(visible_alias = "u")]
    Unlock(ConfigUnlockCmd),
}
