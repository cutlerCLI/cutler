use crate::commands::{GlobalArgs, Runnable};
use anyhow::{Result, anyhow};
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
    /// Initialize a basic config and not the full example.
    #[arg(long)]
    pub basic: bool,

    /// Skip confirmation prompt.
    #[arg(short, long)]
    pub force: bool,
}

#[async_trait]
impl Runnable for InitCmd {
    async fn run(&self, g: &GlobalArgs) -> Result<()> {
        let verbose = g.verbose;
        let dry_run = g.dry_run;
        let config_path = get_config_path();

        let exists = fs::metadata(&config_path).await.is_ok();
        if exists && !self.force {
            print_log(
                LogLevel::Warning,
                &format!("Configuration file already exists at {:?}", config_path),
            );
            if !confirm_action("Do you want to overwrite it?")? {
                return Err(anyhow!("Configuration initialization aborted."));
            }
        }

        // ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            if dry_run {
                print_log(
                    LogLevel::Dry,
                    &format!("Would create directory: {:?}", parent),
                );
            } else {
                if verbose {
                    print_log(
                        LogLevel::Info,
                        &format!("Creating parent dir: {:?}", parent),
                    );
                }
                fs::create_dir_all(parent).await?;
            }
        }

        // default TOML template
        let default_cfg = match self.basic {
            true => {
                if verbose {
                    print_log(LogLevel::Info, "Choosing basic configuration...")
                }
                include_str!("../../examples/basic.toml")
            }
            _ => {
                if verbose {
                    print_log(
                        LogLevel::Info,
                        "No `--basic` flag, defaulting to advanced configuration...",
                    )
                }
                include_str!("../../examples/advanced.toml")
            }
        };

        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would write configuration to {:?}", config_path),
            );
            print_log(
                LogLevel::Dry,
                &format!("Configuration content:\n{}", default_cfg),
            );
        } else {
            fs::write(&config_path, default_cfg).await.map_err(|e| {
                anyhow!("Failed to write configuration to {:?}: {}", config_path, e)
            })?;

            if verbose {
                print_log(
                    LogLevel::Success,
                    &format!("Configuration file created at: {:?}", config_path),
                );
            } else {
                print_log(
                    LogLevel::Fruitful,
                    &format!(
                        "New configuration created at {:?}\nReview and customize this file before running cutler again.",
                        config_path
                    ),
                );
            }
        }

        Ok(())
    }
}
