// SPDX-License-Identifier: MIT

use super::get_styles;
use clap::{Parser, Subcommand};

use crate::commands::{
    ApplyCmd, BrewBackupCmd, BrewInstallCmd, CheckUpdateCmd, CompletionCmd, ConfigDeleteCmd,
    ConfigLockCmd, ConfigShowCmd, ConfigUnlockCmd, ExecCmd, FetchCmd, InitCmd, ResetCmd,
    SelfUpdateCmd, StatusCmd, UnapplyCmd,
};

#[derive(Parser)]
#[command(name = "cutler", styles = get_styles(), version, about)]
pub struct Args {
    /// Increase output verbosity.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors/warnings.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Do not restart system services after execution.
    #[arg(short, long, global = true)]
    pub no_restart_services: bool,

    /// Do not sync with remote (if autosync = true).
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
    /// Apply preferences and more from config.
    #[command(visible_alias = "set")]
    Apply(ApplyCmd),
    /// Run one/all external command(s).
    #[command(visible_alias = "x")]
    Exec(ExecCmd),
    /// Initialize a new config file.
    Init(InitCmd),
    /// Unapply previously applied modifications(s).
    #[command(visible_alias = "undo")]
    Unapply(UnapplyCmd),
    /// (Risky) Hard-reset all preferences.
    Reset(ResetCmd),
    /// Compare your system against config.
    #[command(visible_alias = "s")]
    Status(StatusCmd),
    /// Homebrew-related commands.
    Brew {
        #[command(subcommand)]
        command: BrewSubcmd,
    },
    /// Configuration-related commands.
    #[command(visible_alias = "conf")]
    Config {
        #[command(subcommand)]
        command: ConfigSubcmd,
    },
    /// Check for version updates.
    #[command(visible_alias = "cup")]
    CheckUpdate(CheckUpdateCmd),
    /// Updates cutler itself (for manual installs).
    #[command(visible_alias = "sup")]
    SelfUpdate(SelfUpdateCmd),
    /// Generate shell completions.
    #[command(visible_alias = "comp")]
    Completion(CompletionCmd),
    /// Sync the local config with remote (if any in [remote])
    #[command(visible_alias = "get")]
    Fetch(FetchCmd),
}

#[derive(Subcommand, Debug)]
pub enum BrewSubcmd {
    /// Backup current formulae/casks/taps into config.
    Backup(BrewBackupCmd),
    /// Install formulae/casks/taps from config.
    #[command(visible_alias = "apply")]
    Install(BrewInstallCmd),
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcmd {
    /// Display config contents.
    Show(ConfigShowCmd),
    /// Delete config file.
    #[command(visible_alias = "del")]
    Delete(ConfigDeleteCmd),
    /// Lock config file.
    Lock(ConfigLockCmd),
    /// Unlock config file.
    Unlock(ConfigUnlockCmd),
}
