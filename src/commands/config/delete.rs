use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    commands::{Runnable, UnapplyCmd},
    config::path::get_config_path,
    snapshot::{Snapshot, get_snapshot_path},
    util::{
        globals::should_dry_run,
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};
use anyhow::Result;

#[derive(Debug, Default, Args)]
pub struct ConfigDeleteCmd;

#[async_trait]
impl Runnable for ConfigDeleteCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path().await;
        let dry_run = should_dry_run();

        if !fs::try_exists(&config_path).await.unwrap() {
            print_log(LogLevel::Info, "No config file to delete.");
            return Ok(());
        }

        // offer user to unapply settings if any had been previously applied
        // (use snapshot to detect)
        let snapshot_path = get_snapshot_path();
        if fs::try_exists(&snapshot_path).await.unwrap() {
            print_log(
                LogLevel::Info,
                &format!(
                    "Found a snapshot at {:?}. It contains {} settings.",
                    snapshot_path,
                    Snapshot::load(&snapshot_path).await?.settings.len()
                ),
            );
            if confirm_action("Unapply all previously applied defaults?") {
                UnapplyCmd.run().await?;
            }
        }

        // finally, delete config and snapshot
        if dry_run {
            print_log(LogLevel::Dry, &format!("Would delete {config_path:?}"));
            if fs::try_exists(&snapshot_path).await.unwrap() {
                print_log(LogLevel::Dry, &format!("Would delete {snapshot_path:?}"));
            }
        } else {
            fs::remove_file(&config_path).await?;
            print_log(
                LogLevel::Fruitful,
                &format!("Deleted config at {config_path:?}"),
            );
            if fs::try_exists(&snapshot_path).await.unwrap() {
                fs::remove_file(&snapshot_path).await?;
                print_log(
                    LogLevel::Info,
                    &format!("Deleted snapshot at {snapshot_path:?}"),
                );
            }
        }

        Ok(())
    }
}
