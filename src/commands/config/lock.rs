// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Args;

use anyhow::{Result, bail};

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::core::Config,
    util::logging::{LogLevel, print_log},
};

#[derive(Debug, Args)]
pub struct ConfigLockCmd;

#[async_trait]
impl Runnable for ConfigLockCmd {
    async fn run(&self) -> Result<()> {
        let mut config = Config::load().await?;

        if !Config::is_loadable().await {
            bail!("Cannot find a configuration to lock in the first place.")
        }

        let dry_run = should_dry_run();

        if matches!(config.lock, Some(true)) {
            bail!("Already locked.");
        } else if dry_run {
            print_log(LogLevel::Dry, "Would lock config file.");
            return Ok(());
        }

        config.lock = Some(true);
        config.save().await?;

        Ok(())
    }
}
