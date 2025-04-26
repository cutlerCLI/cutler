pub mod apply;
pub mod config_delete;
pub mod config_show;
pub mod exec;
pub mod init;
pub mod reset;
pub mod status;
pub mod unapply;
pub mod update;

use crate::cli::Command;
use anyhow::Result;

/// Entrypoint: dispatch to each sub‐module’s `run(...)`
pub fn dispatch(command: &Command, verbose: bool, dry_run: bool, no_restart: bool) -> Result<()> {
    let result = match command {
        Command::Apply { no_exec } => apply::run(*no_exec, verbose, dry_run),
        Command::Exec => exec::run(verbose, dry_run),
        Command::Init { force } => init::run(verbose, *force),
        Command::Unapply => unapply::run(verbose, dry_run),
        Command::Reset { force } => reset::run(verbose, dry_run, *force),
        Command::Status { prompt } => status::run(*prompt, verbose),
        Command::Config { command } => match command {
            crate::cli::ConfigSub::Show => config_show::run(verbose, dry_run),
            crate::cli::ConfigSub::Delete => config_delete::run(verbose, dry_run),
        },
        Command::Completion { shell } => crate::cli::completion::generate_completion(*shell),
        Command::CheckUpdate => update::run(verbose),
    };

    // handle post‐hooks (restart services)
    if result.is_ok() {
        use crate::util::io::restart_system_services;
        match command {
            Command::Apply { .. }
            | Command::Unapply
            | Command::Reset { .. }
            | Command::Config {
                command: crate::cli::ConfigSub::Delete,
            } => {
                if !no_restart {
                    let _ = restart_system_services(verbose, dry_run);
                }
            }
            _ => {}
        }
    }

    result
}
