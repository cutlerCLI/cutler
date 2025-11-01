// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;

use crate::{
    commands::Runnable,
    config::core::{Config, Mas},
    log_cute, log_info, mas,
};

#[derive(Args, Debug)]
pub struct MasBackupCmd;

#[async_trait]
impl Runnable for MasBackupCmd {
    async fn run(&self) -> Result<()> {
        if !Config::is_loadable().await {
            bail!("Cannot run command since config could not be loaded.")
        }

        let mut config = Config::load(true).await?;

        let listed_apps: Vec<String> = mas::list_apps()
            .await?
            .into_iter()
            .map(|item| {
                log_info!("Pushing app: {} ({})", item.id, item.name);
                item.id
            })
            .collect();

        if listed_apps.is_empty() {
            log_cute!("Nothing to backup!");
        }

        let mas_table = Mas { ids: listed_apps };
        log_info!("Modifying table for existing configuration with backup: {mas_table:?}");
        config.mas = Some(mas_table);

        config.save().await?;

        Ok(())
    }
}
