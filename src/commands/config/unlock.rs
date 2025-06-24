use async_trait::async_trait;
use clap::Args;

use anyhow::{Context, Result, bail};
use tokio::fs;
use toml_edit::{DocumentMut, Item};

use crate::{
    commands::Runnable,
    config::get_config_path,
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
        let cfg_path = get_config_path();
        let dry_run = should_dry_run();

        let mut doc = if cfg_path.exists() {
            let text = fs::read_to_string(&cfg_path).await?;
            text.parse::<DocumentMut>()
                .context("Failed to parse config TOML!")?
        } else {
            bail!("Cannot lock a config file that does not exist.")
        };

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
