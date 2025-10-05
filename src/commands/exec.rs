// SPDX-License-Identifier: Apache-2.0

use crate::commands::Runnable;

use crate::config::loader::Config;
use crate::exec::runner;
use crate::exec::runner::ExecMode;
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

#[derive(Args, Debug)]
pub struct ExecCmd {
    /// The command to execute. Defaults to 'all' if not passed.
    #[arg(value_name = "NAME")]
    name: Option<String>,

    /// Execute in regular mode (no flagged commands).
    #[arg(short, long, conflicts_with = "flagged")]
    regular: bool,

    /// Execute flagged commands only.
    #[arg(short, long, conflicts_with = "regular")]
    flagged: bool,
}

#[async_trait]
impl Runnable for ExecCmd {
    async fn run(&self) -> Result<()> {
        // load & parse config
        let config = Config::load().await?;

        let mode = if self.flagged {
            ExecMode::Flagged
        } else if self.regular {
            ExecMode::Regular
        } else {
            ExecMode::All
        };

        if let Some(cmd_name) = &self.name {
            runner::run_one(config, cmd_name).await?;
        } else {
            runner::run_all(config, mode).await?;
        }

        Ok(())
    }
}
