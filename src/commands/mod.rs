use anyhow::Result;
use async_trait::async_trait;

pub mod apply;
pub mod brew_backup;
pub mod brew_install;
pub mod config_delete;
pub mod config_show;
pub mod exec;
pub mod init;
pub mod reset;
pub mod status;
pub mod unapply;
pub mod update;

// Re-export command structs for CLI usage
pub use apply::ApplyCmd;
pub use brew_backup::BrewBackupCmd;
pub use brew_install::BrewInstallCmd;
pub use config_delete::ConfigDeleteCmd;
pub use config_show::ConfigShowCmd;
pub use exec::ExecCmd;
pub use init::InitCmd;
pub use reset::ResetCmd;
pub use status::StatusCmd;
pub use unapply::UnapplyCmd;
pub use update::{CheckUpdateCmd, SelfUpdateCmd};

/// Struct holding all global CLI flags to be passed to commands
#[derive(Debug, Clone, Copy)]
pub struct GlobalArgs {
    pub verbose: bool,
    pub dry_run: bool,
    pub quiet: bool,
    pub no_restart_services: bool,
    pub accept_interactive: bool,
}

/// Trait for all runnable commands.
#[async_trait]
pub trait Runnable {
    async fn run(&self, globals: &GlobalArgs) -> Result<()>;
}
