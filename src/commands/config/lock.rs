use async_trait::async_trait;
use clap::Args;

use anyhow::{Result, bail};
use tokio::fs;
use toml_edit::Item;

use crate::{
    commands::Runnable,
    config::loader::{get_config_path, load_config_mut},
    util::{
        globals::should_dry_run,
        logging::{LogLevel, print_log},
    },
};

#[derive(Debug, Default, Args)]
pub struct ConfigLockCmd;

#[async_trait]
impl Runnable for ConfigLockCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path().await;
        let dry_run = should_dry_run();

        let mut doc = load_config_mut(false).await?;
        let is_locked = doc.get("lock").and_then(Item::as_bool).unwrap_or(false);

        if is_locked {
            bail!("Already locked.");
        } else if dry_run {
            print_log(LogLevel::Dry, "Would lock config file.");
            return Ok(());
        }

        doc["lock"] = true.into();
        fs::write(&cfg_path, doc.to_string()).await?;

        Ok(())
    }
}
