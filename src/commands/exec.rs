// SPDX-License-Identifier: GPL-3.0-or-later

use crate::commands::Runnable;

use crate::config::core::Config;
use crate::exec::core;
use crate::exec::core::ExecMode;
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

#[derive(Args, Debug)]
pub struct ExecCmd {
    /// The command to execute. Defaults to 'all' if not passed.
    #[arg(value_name = "NAME")]
    name: Option<String>,

    /// Executes all declared commands.
    #[arg(short, long, conflicts_with = "flagged")]
    all: bool,

    /// Execute flagged commands only.
    #[arg(short, long, conflicts_with = "all")]
    flagged: bool,
}

#[async_trait]
impl Runnable for ExecCmd {
    async fn run(&self) -> Result<()> {
        // load & parse config
        let config = Config::load(true).await?;

        let mode = if self.all {
            ExecMode::All
        } else if self.flagged {
            ExecMode::Flagged
        } else {
            ExecMode::Regular
        };

        if let Some(cmd_name) = &self.name {
            core::run_one(config, cmd_name).await?;
        } else {
            core::run_all(config, mode).await?;
        }

        Ok(())
    }
}
