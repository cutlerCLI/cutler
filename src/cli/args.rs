// SPDX-License-Identifier: MIT OR Apache-2.0

use clap::{Parser, Subcommand};

use crate::commands::{
    ApplyCmd, BrewBackupCmd, BrewInstallCmd, CheckUpdateCmd, CompletionCmd, ConfigCmd, CookbookCmd,
    ExecCmd, FetchCmd, InitCmd, LockCmd, ResetCmd, Runnable, SelfUpdateCmd, StatusCmd, UnapplyCmd,
    UnlockCmd,
};

#[derive(Parser)]
#[command(
    name = "cutler",
    version,
    about,
    help_template = "\
{name} {version}
{about}

usage
{usage}

options:
{options}

subcmds:
{subcommands}

For more information on a subcommand, run `{name} <subcommand> --help`
    "
)]
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
    /// Open the cookbook for cutler in browser.
    #[command(visible_alias = "doc")]
    Cookbook(CookbookCmd),
    /// Run one/all external command(s).
    #[command(visible_alias = "x")]
    Exec(ExecCmd),
    /// Initialize a new config file.
    Init(InitCmd),
    /// Lock the config.
    Lock(LockCmd),
    /// Unlock the config.
    Unlock(UnlockCmd),
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
    /// Shows the configuration.
    #[command(visible_alias = "conf")]
    Config(ConfigCmd),
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

impl Command {
    /// Returns a trait object reference for a given command so that it can
    /// be run using the .run() implementation of that particular command.
    pub fn as_runnable(&self) -> &dyn Runnable {
        match self {
            Command::Apply(cmd) => cmd,
            Command::Config(cmd) => cmd,
            Command::Cookbook(cmd) => cmd,
            Command::Exec(cmd) => cmd,
            Command::Fetch(cmd) => cmd,
            Command::Init(cmd) => cmd,
            Command::Unapply(cmd) => cmd,
            Command::Reset(cmd) => cmd,
            Command::Status(cmd) => cmd,
            Command::Lock(cmd) => cmd,
            Command::Unlock(cmd) => cmd,
            Command::CheckUpdate(cmd) => cmd,
            Command::SelfUpdate(cmd) => cmd,
            Command::Completion(cmd) => cmd,
            Command::Brew { command } => match command {
                BrewSubcmd::Backup(cmd) => cmd as &dyn Runnable,
                BrewSubcmd::Install(cmd) => cmd as &dyn Runnable,
            },
        }
    }
}
