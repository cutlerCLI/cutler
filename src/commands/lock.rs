// SPDX-License-Identifier: MIT OR Apache-2.0

use async_trait::async_trait;
use clap::Args;

use anyhow::{Result, bail};

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::core::{Config, ConfigCoreMethods},
    log_dry,
};

#[derive(Debug, Args)]
pub struct LockCmd;

#[async_trait]
impl Runnable for LockCmd {
    async fn run(&self, config: &mut Config) -> Result<()> {
        if !config.is_loadable() {
            bail!("Cannot find a configuration to lock in the first place.")
        }

        let mut document = config.load_as_mut(false).await?;
        let dry_run = should_dry_run();

        if document
            .get("lock")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            bail!("Already locked.");
        } else if dry_run {
            log_dry!("Would lock config file.");
            return Ok(());
        }

        document["lock"] = toml_edit::value(true);
        document.save(&config.path).await?;

        Ok(())
    }
}
