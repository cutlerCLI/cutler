// SPDX-License-Identifier: MIT OR Apache-2.0

use async_trait::async_trait;
use clap::Args;

use anyhow::{Result, bail};

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{core::Config, path::get_config_path},
    log_dry,
};

#[derive(Debug, Args)]
pub struct UnlockCmd;

#[async_trait]
impl Runnable for UnlockCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path().await?;

        if !config_path.try_exists()? {
            bail!("Cannot find a configuration to unlock in the first place.")
        }

        let config = Config::new(config_path);
        let mut document = config.load_as_mut(false).await?;
        let dry_run = should_dry_run();

        if !document
            .get("lock")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            bail!("Already unlocked.")
        } else if dry_run {
            log_dry!("Would unlock config file.");
            return Ok(());
        }

        document.remove("lock");
        config.save(Some(document)).await?;

        Ok(())
    }
}
