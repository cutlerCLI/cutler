use anyhow::Result;
use async_trait::async_trait;

pub mod apply;
pub mod brew;
pub mod check_update;
pub mod completion;
pub mod config;
pub mod exec;
pub mod fetch;
pub mod init;
pub mod reset;
pub mod self_update;
pub mod status;
pub mod unapply;

pub use apply::ApplyCmd;
pub use brew::backup::BrewBackupCmd;
pub use brew::install::BrewInstallCmd;
pub use check_update::CheckUpdateCmd;
pub use completion::{CompletionCmd, Shell};
pub use config::delete::ConfigDeleteCmd;
pub use config::show::ConfigShowCmd;
pub use exec::ExecCmd;
pub use fetch::FetchCmd;
pub use init::InitCmd;
pub use reset::ResetCmd;
pub use self_update::SelfUpdateCmd;
pub use status::StatusCmd;
pub use unapply::UnapplyCmd;

/// Trait for all runnable commands.
#[async_trait]
pub trait Runnable {
    async fn run(&self) -> Result<()>;
}
