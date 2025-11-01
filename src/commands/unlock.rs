// SPDX-License-Identifier: GPL-3.0-or-later

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
pub struct UnlockCmd;

#[async_trait]
impl Runnable for UnlockCmd {
    async fn run(&self) -> Result<()> {
        if !Config::is_loadable().await {
            bail!("Cannot find a configuration to unlock in the first place.")
        }

        let mut config = Config::load(false).await?;
        let dry_run = should_dry_run();

        if config.lock.is_none_or(|val| !val) {
            bail!("Already unlocked.")
        } else if dry_run {
            print_log(LogLevel::Dry, "Would unlock config file.");
            return Ok(());
        }

        config.lock = None;
        config.save().await?;

        Ok(())
    }
}
