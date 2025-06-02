use tokio::fs;

use crate::{
    commands::unapply,
    config::loader::get_config_path,
    snapshot::{Snapshot, get_snapshot_path},
    util::{
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};
use anyhow::Result;

pub async fn run(verbose: bool, dry_run: bool) -> Result<()> {
    let config_path = get_config_path();
    if !config_path.exists() {
        if verbose {
            print_log(LogLevel::Success, "No config file to delete.");
        }
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
            unapply::run(verbose, dry_run).await?;
        }
    }

    // finally, delete config and snapshot
    if dry_run {
        print_log(
            LogLevel::Dry,
            &format!("Would delete {:?}", config_path),
        );
        if snapshot_path.exists() {
            print_log(
                LogLevel::Dry,
                &format!("Would delete {:?}", snapshot_path),
            );
        }
    } else {
        fs::remove_file(&config_path).await?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Deleted config at {:?}", config_path),
            );
        }
        if snapshot_path.exists() {
            fs::remove_file(&snapshot_path).await?;
            if verbose {
                print_log(
                    LogLevel::Success,
                    &format!("Deleted snapshot at {:?}", snapshot_path),
                );
            }
        }
    }

    Ok(())
}
