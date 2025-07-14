use crate::commands::Runnable;

use crate::{config::loader::load_config, exec::runner};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

#[derive(Args, Debug)]
pub struct ExecCmd {
    /// Provide a command name to execute if you only want to run it specifically.
    #[arg(value_name = "NAME")]
    pub name: Option<String>,
}

#[async_trait]
impl Runnable for ExecCmd {
    async fn run(&self) -> Result<()> {
        // load & parse config
        let toml = load_config(true).await?;

        if let Some(cmd_name) = &self.name {
            runner::run_one(&toml, cmd_name).await?;
        } else {
            runner::run_all(&toml).await?;
        }

        Ok(())
    }
}
