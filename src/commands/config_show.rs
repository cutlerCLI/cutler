use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    commands::{GlobalArgs, Runnable},
    config::loader::get_config_path,
    util::logging::{LogLevel, print_log},
};

#[derive(Debug, Default, Args)]
pub struct ConfigShowCmd;

#[async_trait]
impl Runnable for ConfigShowCmd {
    async fn run(&self, g: &GlobalArgs) -> Result<()> {
        let config_path = get_config_path();
        let verbose = g.verbose;
        let dry_run = g.dry_run;
        let quiet = g.quiet;

        if !config_path.exists() {
            bail!("Configuration file does not exist at {:?}", config_path);
        }

        // handle dry‑run
        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would display config at {:?}", config_path),
            );
            return Ok(());
        }

        // read and print the file
        let content = fs::read_to_string(&config_path).await?;
        if !quiet {
            println!("{}", content);
        }

        if verbose {
            print_log(LogLevel::Info, "Displayed configuration file.");
        }

        Ok(())
    }
}
