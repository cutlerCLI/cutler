// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    commands::Runnable,
    config::{core::Config, path::get_config_path},
    log,
    util::{io::confirm, logging::LogLevel},
};

#[derive(Args, Debug)]
pub struct InitCmd;

#[async_trait]
impl Runnable for InitCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path().await?;

        if Config::is_loadable().await {
            log!(
                LogLevel::Warning,
                "Configuration file already exists at {config_path:?}",
            );
            if !confirm("Do you want to overwrite it?") {
                bail!("Configuration init aborted.")
            }
        }

        // write TOML template to disk
        // this is not done by create_empty_config
        let default_cfg = include_str!("../../examples/complete.toml");

        fs::create_dir_all(config_path.parent().unwrap()).await?;
        fs::write(&config_path, default_cfg).await?;

        log!(
            LogLevel::Fruitful,
            "Config created at {config_path:?}, Review and customize it before applying.",
        );

        Ok(())
    }
}
