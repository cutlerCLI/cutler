// SPDX-License-Identifier: Apache-2.0

use crate::commands::Runnable;

use crate::exec::runner::ExecMode;
use crate::{config::loader::load_config, exec::runner};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

#[derive(Args, Debug)]
pub struct ExecCmd {
    /// The command to execute. Defaults to 'all' if not passed.
    #[arg(value_name = "NAME")]
    pub name: Option<String>,

    /// Execute all commands.
    #[arg(short, long, conflicts_with = "flagged")]
    pub all: bool,

    /// Execute flagged commands only.
    #[arg(short, long, conflicts_with = "all")]
    pub flagged: bool,
}

#[async_trait]
impl Runnable for ExecCmd {
    async fn run(&self) -> Result<()> {
        // load & parse config
        let toml = load_config(true).await?;

        let mode = if self.all {
            ExecMode::All
        } else if self.flagged {
            ExecMode::Flagged
        } else {
            ExecMode::Regular
        };

        if let Some(cmd_name) = &self.name {
            runner::run_one(&toml, cmd_name).await?;
        } else {
            runner::run_all(&toml, mode).await?;
        }

        Ok(())
    }
}
