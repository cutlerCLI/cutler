use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    commands::{Runnable, UnapplyCmd},
    config::loader::get_config_path,
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
        let config_path = get_config_path();
        let dry_run = should_dry_run();

        if !config_path.exists() {
            print_log(LogLevel::Success, "No config file to delete.");
            return Ok(());
        }

        // offer user to unapply settings if any had been previously applied
        // (use snapshot to detect)
        let snapshot_path = get_snapshot_path();
        if snapshot_path.exists() {
            println!(
                "Found a snapshot at {:?}. It contains {} settings.",
                snapshot_path,
                Snapshot::load(&snapshot_path).await?.settings.len()
            );
            if confirm_action("Unapply all previously applied defaults?")? {
                UnapplyCmd.run().await?;
            }
        }

        // finally, delete config and snapshot
        if dry_run {
            print_log(LogLevel::Dry, &format!("Would delete {:?}", config_path));
            if snapshot_path.exists() {
                print_log(LogLevel::Dry, &format!("Would delete {:?}", snapshot_path));
            }
        } else {
            fs::remove_file(&config_path).await?;
            print_log(
                LogLevel::Success,
                &format!("Deleted config at {:?}", config_path),
            );
            if snapshot_path.exists() {
                fs::remove_file(&snapshot_path).await?;
                print_log(
                    LogLevel::Success,
                    &format!("Deleted snapshot at {:?}", snapshot_path),
                );
            }
        }

        Ok(())
    }
}
