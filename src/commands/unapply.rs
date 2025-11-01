// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, Preferences};
use std::collections::HashMap;

use crate::{
    cli::atomic::should_dry_run,
    commands::{ResetCmd, Runnable},
    config::core::Config,
    domains::convert::{string_to_toml_value, toml_to_prefvalue},
    log_cute, log_dry, log_err, log_info, log_warn,
    snapshot::{core::Snapshot, get_snapshot_path},
    util::{
        io::{confirm, restart_services},
        sha::get_digest,
    },
};

#[derive(Args, Debug)]
pub struct UnapplyCmd;

#[async_trait]
impl Runnable for UnapplyCmd {
    async fn run(&self) -> Result<()> {
        let config = Config::load(true).await?;

        if !Snapshot::is_loadable().await {
            log_warn!("No snapshot found to revert.");

            if confirm("Reset all System Settings instead?") {
                return ResetCmd.run().await;
            } else {
                bail!("Abort operation.")
            }
        }

        let dry_run = should_dry_run();

        // load snapshot from disk
        let snap_path = get_snapshot_path().await?;
        let snapshot = match Snapshot::load(&snap_path).await {
            Ok(snap) => snap,
            Err(_) => {
                bail!(
                    "Could not read snapshot since it might be corrupt. \n\
                    Use `cutler reset` instead to return System Settings to factory defaults."
                )
            }
        };

        if snapshot.digest != get_digest(config.path)? {
            log_warn!("Config has been modified since last application.",);
            log_warn!("Please note that only the applied modifications will be unapplied.",);
        }

        // prepare undo operations, grouping by domain for efficiency
        let mut batch_restores: HashMap<Domain, Vec<(String, defaults_rs::PrefValue)>> =
            HashMap::new();
        let mut batch_deletes: HashMap<Domain, Vec<String>> = HashMap::new();

        // reverse order to undo in correct sequence
        for s in snapshot.settings.clone().into_iter().rev() {
            let domain_obj = if s.domain == "NSGlobalDomain" {
                Domain::Global
            } else {
                Domain::User(s.domain.clone())
            };
            if let Some(orig) = s.original_value {
                let pref_value = toml_to_prefvalue(&string_to_toml_value(&orig))?;
                batch_restores
                    .entry(domain_obj)
                    .or_default()
                    .push((s.key, pref_value));
            } else {
                batch_deletes.entry(domain_obj).or_default().push(s.key);
            }
        }

        // in dry-run mode, just print what would be done
        if dry_run {
            for (domain, restores) in &batch_restores {
                for (key, value) in restores {
                    log_dry!("Would restore: {domain} | {key} -> {value}",);
                }
            }
            for (domain, deletes) in &batch_deletes {
                for key in deletes {
                    log_dry!("Would delete setting: {domain} | {key}",);
                }
            }
        } else {
            // perform batch restores
            if !batch_restores.is_empty() {
                let mut batch_vec = Vec::new();
                for (domain, entries) in batch_restores {
                    for (key, value) in entries {
                        log_info!("Restoring: {domain} | {key} -> {value}",);
                        batch_vec.push((domain.clone(), key, value));
                    }
                }
                if let Err(e) = Preferences::write_batch(batch_vec.clone()).await {
                    log_err!("Batch restore failed: {e}");
                }
            }

            // perform batch deletes
            if !batch_deletes.is_empty() {
                let mut delete_vec = Vec::new();
                for (domain, keys) in batch_deletes {
                    for key in keys {
                        log_info!("Deleting: {domain} | {key}");
                        delete_vec.push((domain.clone(), Some(key)));
                    }
                }
                if let Err(e) = Preferences::delete_batch(delete_vec.clone()).await {
                    log_err!("Batch delete failed: {e}");
                }
            }
        }

        // warn about external command execution
        if snapshot.exec_run_count > 0 {
            log_warn!(
                "{} commands were executed previously; revert them manually.",
                snapshot.exec_run_count
            );
        }

        // delete the snapshot file
        if dry_run {
            log_dry!("Would remove snapshot file at {snap_path:?}",);
        } else {
            snapshot.delete().await?;
            log_info!("Removed snapshot file at {snap_path:?}",);
        }

        // Restart system services if requested
        restart_services().await;

        log_cute!("Unapply operation complete.");

        Ok(())
    }
}
