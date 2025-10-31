// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, Preferences};
use tokio::fs;

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::core::Config,
    domains::{collect, effective, read_current},
    log,
    snapshot::{Snapshot, get_snapshot_path},
    util::{
        io::{confirm, restart_services},
        logging::LogLevel,
    },
};

#[derive(Args, Debug)]
pub struct ResetCmd;

#[async_trait]
impl Runnable for ResetCmd {
    async fn run(&self) -> Result<()> {
        let dry_run = should_dry_run();
        let config = Config::load(true).await?;

        log!(
            LogLevel::Warning,
            "This will DELETE all settings defined in your config file.",
        );
        log!(
            LogLevel::Warning,
            "Settings will be reset to macOS defaults, not to their previous values.",
        );

        if !confirm("Are you sure you want to continue?") {
            return Ok(());
        }

        let domains = collect(&config)?;

        for (domain, table) in domains {
            for (key, _) in table {
                let (eff_dom, eff_key) = effective(&domain, &key);

                // only delete it if currently set
                if read_current(&eff_dom, &eff_key).await.is_some() {
                    let domain_obj = if eff_dom == "NSGlobalDomain" {
                        Domain::Global
                    } else if let Some(rest) = eff_dom.strip_prefix("com.apple.") {
                        Domain::User(format!("com.apple.{rest}"))
                    } else {
                        Domain::User(eff_dom.clone())
                    };

                    if dry_run {
                        log!(
                            LogLevel::Dry,
                            "Would reset {eff_dom}.{eff_key} to system default",
                        );
                    } else {
                        match Preferences::delete(domain_obj, Some(&eff_key)).await {
                            Ok(_) => {
                                log!(
                                    LogLevel::Info,
                                    "Reset {eff_dom}.{eff_key} to system default",
                                );
                            }
                            Err(e) => {
                                log!(LogLevel::Error, "Failed to reset {eff_dom}.{eff_key}: {e}",);
                            }
                        }
                    }
                } else {
                    log!(LogLevel::Info, "Skipping {eff_dom}.{eff_key} (not set)",);
                }
            }
        }

        // remove snapshot if present
        let snap_path = get_snapshot_path().await?;
        if Snapshot::is_loadable().await {
            if dry_run {
                log!(LogLevel::Dry, "Would remove snapshot at {snap_path:?}",);
            } else if let Err(e) = fs::remove_file(&snap_path).await {
                log!(LogLevel::Warning, "Failed to remove snapshot: {e}",);
            } else {
                log!(LogLevel::Info, "Removed snapshot at {snap_path:?}",);
            }
        }

        log!(
            LogLevel::Fruitful,
            "Reset complete. All configured settings have been removed.",
        );

        // restart system services if requested
        restart_services().await;

        log!(LogLevel::Fruitful, "Reset operation complete.");

        Ok(())
    }
}
