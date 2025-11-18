// SPDX-License-Identifier: MIT OR Apache-2.0

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
    log_cute, log_dry, log_err, log_info, log_warn,
    snapshot::{Snapshot, get_snapshot_path},
    util::io::{confirm, restart_services},
};

#[derive(Args, Debug)]
pub struct ResetCmd;

#[async_trait]
impl Runnable for ResetCmd {
    async fn run(&self, config: &mut Config) -> Result<()> {
        let dry_run = should_dry_run();

        config.load(true).await?;

        log_warn!("This will DELETE all settings defined in your config file.",);
        log_warn!("Settings will be reset to macOS defaults, not to their previous values.",);

        if !confirm("Are you sure you want to continue?") {
            return Ok(());
        }

        let domains = collect(&config).await?;

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
                        log_dry!("Would reset {eff_dom}.{eff_key} to system default",);
                    } else {
                        match Preferences::delete(domain_obj, &eff_key) {
                            Ok(_) => {
                                log_info!("Reset {eff_dom}.{eff_key} to system default",);
                            }
                            Err(e) => {
                                log_err!("Failed to reset {eff_dom}.{eff_key}: {e}",);
                            }
                        }
                    }
                } else {
                    log_info!("Skipping {eff_dom}.{eff_key} (not set)",);
                }
            }
        }

        // remove snapshot if present
        let snap_path = get_snapshot_path().await?;
        if Snapshot::is_loadable().await {
            if dry_run {
                log_dry!("Would remove snapshot at {snap_path:?}",);
            } else if let Err(e) = fs::remove_file(&snap_path).await {
                log_warn!("Failed to remove snapshot: {e}",);
            } else {
                log_info!("Removed snapshot at {snap_path:?}",);
            }
        }

        log_cute!("Reset complete. All configured settings have been removed.",);

        // restart system services if requested
        restart_services().await;

        log_cute!("Reset operation complete.");

        Ok(())
    }
}
