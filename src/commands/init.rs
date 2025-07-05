use crate::{commands::Runnable, config::loader::create_config};
use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    config::loader::get_config_path,
    util::{
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};

/// Initialize a new config file with sensible defaults.
#[derive(Args, Debug)]
pub struct InitCmd {
    /// Skip confirmation prompt.
    #[arg(short, long)]
    pub force: bool,
}

#[async_trait]
impl Runnable for InitCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path().await;

        let exists = fs::metadata(&config_path).await.is_ok();
        if exists {
            print_log(
                LogLevel::Warning,
                &format!("Configuration file already exists at {config_path:?}"),
            );
            if !confirm_action("Do you want to overwrite it?")? {
                bail!("Configuration init aborted.")
            } else {
                create_config(&config_path).await?;
            }
        }
        Ok(())
    }
}
