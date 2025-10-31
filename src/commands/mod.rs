// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;

pub mod apply;
pub mod brew;
pub mod check_update;
pub mod completion;
pub mod config;
pub mod cookbook;
pub mod exec;
pub mod fetch;
pub mod init;
pub mod lock;
pub mod mas;
pub mod reset;
pub mod self_update;
pub mod status;
pub mod unapply;
pub mod unlock;

// this is directly used by src/cli/args.rs and other parts of the code to match commands
pub use apply::ApplyCmd;
pub use brew::{backup::BrewBackupCmd, install::BrewInstallCmd};
pub use check_update::CheckUpdateCmd;
pub use completion::CompletionCmd;
pub use config::ConfigCmd;
pub use cookbook::CookbookCmd;
pub use exec::ExecCmd;
pub use fetch::FetchCmd;
pub use init::InitCmd;
pub use lock::LockCmd;
pub use mas::list::MasListCmd;
pub use reset::ResetCmd;
pub use self_update::SelfUpdateCmd;
pub use status::StatusCmd;
pub use unapply::UnapplyCmd;
pub use unlock::UnlockCmd;

/// Trait for all runnable commands.
#[async_trait]
pub trait Runnable {
    async fn run(&self) -> Result<()>;
}
