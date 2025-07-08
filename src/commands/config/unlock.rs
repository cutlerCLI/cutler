use async_trait::async_trait;
use clap::Args;

use anyhow::Result;
use tokio::fs;
use toml_edit::Item;

use crate::{
    commands::Runnable,
    config::{get_config_path, loader::load_config_mut},
    util::{
        globals::should_dry_run,
        logging::{LogLevel, print_log},
    },
};

#[derive(Debug, Default, Args)]
pub struct ConfigUnlockCmd;

#[async_trait]
impl Runnable for ConfigUnlockCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path().await;
        let dry_run = should_dry_run();

        let mut doc = load_config_mut(false).await?;
        let is_locked = doc.get("lock").and_then(Item::as_bool).unwrap_or(false);

        if !is_locked {
            print_log(LogLevel::Fruitful, "Already unlocked.");
            return Ok(());
        } else if dry_run {
            print_log(LogLevel::Dry, "Would unlock config file.");
            return Ok(());
        }

        doc.remove("lock");
        fs::write(&cfg_path, doc.to_string()).await?;

        Ok(())
    }
}
