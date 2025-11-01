// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Args;

use anyhow::{Result, bail};

use crate::{cli::atomic::should_dry_run, commands::Runnable, config::core::Config, log_dry};

#[derive(Debug, Args)]
pub struct LockCmd;

#[async_trait]
impl Runnable for LockCmd {
    async fn run(&self) -> Result<()> {
        if !Config::is_loadable().await {
            bail!("Cannot find a configuration to lock in the first place.")
        }

        let mut config = Config::load(false).await?;
        let dry_run = should_dry_run();

        if matches!(config.lock, Some(true)) {
            bail!("Already locked.");
        } else if dry_run {
            log_dry!("Would lock config file.");
            return Ok(());
        }

        config.lock = Some(true);
        config.save().await?;

        Ok(())
    }
}
