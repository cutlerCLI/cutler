use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;

use crate::{
    commands::Runnable,
    config::core::Config,
    mas,
    util::logging::{LogLevel, print_log},
};

#[derive(Args, Debug)]
pub struct MasListCmd;

#[async_trait]
impl Runnable for MasListCmd {
    async fn run(&self) -> Result<()> {
        if !Config::is_loadable().await {
            bail!("Cannot run command since config could not be loaded.")
        }

        let mas_table = mas::list_apps().await?;

        for item in mas_table {
            print_log(LogLevel::Info, &format!("{item:?}"));
        }

        Ok(())
    }
}
