use anyhow::Result;
use async_trait::async_trait;

pub mod apply;
pub mod brew;
pub mod config;
pub mod exec;
pub mod init;
pub mod reset;
pub mod status;
pub mod unapply;
pub mod update;

// Re-export command structs for CLI usage
pub use apply::ApplyCmd;
pub use brew::backup::BrewBackupCmd;
pub use brew::install::BrewInstallCmd;
pub use config::delete::ConfigDeleteCmd;
pub use config::show::ConfigShowCmd;
pub use exec::ExecCmd;
pub use init::InitCmd;
pub use reset::ResetCmd;
pub use status::StatusCmd;
pub use unapply::UnapplyCmd;
pub use update::{CheckUpdateCmd, SelfUpdateCmd};

/// Trait for all runnable commands.
#[async_trait]
pub trait Runnable {
    async fn run(&self) -> Result<()>;
}
