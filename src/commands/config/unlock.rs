// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use clap::Args;

use anyhow::{Result, bail};
use tokio::fs;
use toml_edit::Item;

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{loader::load_config_mut, path::get_config_path},
    util::logging::{LogLevel, print_log},
};

#[derive(Debug, Default, Args)]
pub struct ConfigUnlockCmd;

#[async_trait]
impl Runnable for ConfigUnlockCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path().await;

        if !fs::try_exists(&cfg_path).await.unwrap() {
            bail!("Cannot find a configuration to unlock in the first place.")
        }

        let dry_run = should_dry_run();

        let mut doc = load_config_mut(false).await?;
        let is_locked = doc.get("lock").and_then(Item::as_bool).unwrap_or(false);

        if !is_locked {
            bail!("Already unlocked.")
        } else if dry_run {
            print_log(LogLevel::Dry, "Would unlock config file.");
            return Ok(());
        }

        doc.remove("lock");
        fs::write(&cfg_path, doc.to_string()).await?;

        Ok(())
    }
}
