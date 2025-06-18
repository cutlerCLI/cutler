use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    commands::Runnable,
    config::loader::get_config_path,
    util::{
        globals::{should_be_quiet, should_dry_run},
        logging::{LogLevel, print_log},
    },
};

#[derive(Debug, Default, Args)]
pub struct ConfigShowCmd;

#[async_trait]
impl Runnable for ConfigShowCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path();

        if !config_path.exists() {
            bail!("Configuration file does not exist at {:?}", config_path);
        }

        // handle dry‑run
        if should_dry_run() {
            print_log(
                LogLevel::Dry,
                &format!("Would display config at {:?}", config_path),
            );
            return Ok(());
        }

        // read and print the file
        let content = fs::read_to_string(&config_path).await?;
        if !should_be_quiet() {
            println!("{}", content);
        }

        print_log(LogLevel::Info, "Displayed configuration file.");

        Ok(())
    }
}
