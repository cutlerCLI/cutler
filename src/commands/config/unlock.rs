// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Args;

use anyhow::{Result, bail};
use tokio::fs;

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{loader::Config, path::get_config_path},
    util::logging::{LogLevel, print_log},
};

#[derive(Debug, Args)]
pub struct ConfigUnlockCmd;

#[async_trait]
impl Runnable for ConfigUnlockCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path().await;

        if !fs::try_exists(&cfg_path).await? {
            bail!("Cannot find a configuration to unlock in the first place.")
        }

        let dry_run = should_dry_run();

        let mut config = Config::load().await?;

        if config.lock.is_none_or(|val| val == false) {
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
