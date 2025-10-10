// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, preferences::Preferences};
use std::collections::HashMap;

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::core::Config,
    domains::convert::{string_to_toml_value, toml_to_prefvalue},
    snapshot::{core::Snapshot, get_snapshot_path},
    util::{
        io::{notify, restart_services},
        logging::{LogLevel, print_log},
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
            bail!(
                "No snapshot found. \n\
                Use `cutler reset` to return System Settings to factory defaults."
            );
        }

        let dry_run = should_dry_run();

        // load snapshot from disk
        let snap_path = get_snapshot_path()?;
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
            print_log(
                LogLevel::Warning,
                "Config has been modified since last application.",
            );
            print_log(
                LogLevel::Warning,
                "Please note that only the applied modifications will be unapplied.",
            );
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
                    print_log(
                        LogLevel::Dry,
                        &format!("Would restore: {domain} | {key} -> {value}"),
                    );
                }
            }
            for (domain, deletes) in &batch_deletes {
                for key in deletes {
                    print_log(
                        LogLevel::Dry,
                        &format!("Would delete setting: {domain} | {key}"),
                    );
                }
            }
        } else {
            // perform batch restores
            if !batch_restores.is_empty() {
                let mut batch_vec = Vec::new();
                for (domain, entries) in batch_restores {
                    for (key, value) in entries {
                        print_log(
                            LogLevel::Info,
                            &format!("Restoring: {domain} | {key} -> {value}"),
                        );
                        batch_vec.push((domain.clone(), key, value));
                    }
                }
                if let Err(e) = Preferences::write_batch(batch_vec.clone()).await {
                    print_log(LogLevel::Error, &format!("Batch restore failed: {e}"));
                }
            }

            // perform batch deletes
            if !batch_deletes.is_empty() {
                let mut delete_vec = Vec::new();
                for (domain, keys) in batch_deletes {
                    for key in keys {
                        print_log(LogLevel::Info, &format!("Deleting: {domain} | {key}"));
                        delete_vec.push((domain.clone(), Some(key)));
                    }
                }
                if let Err(e) = Preferences::delete_batch(delete_vec.clone()).await {
                    print_log(LogLevel::Error, &format!("Batch delete failed: {e}"));
                }
            }
        }

        // warn about external command execution
        if snapshot.exec_run_count > 0 {
            print_log(
                LogLevel::Warning,
                &format!(
                    "{} External commands were executed previously; please revert them manually if needed.",
                    snapshot.exec_run_count
                ),
            );
        }

        // delete the snapshot file
        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would remove snapshot file at {snap_path:?}"),
            );
        } else {
            snapshot.delete().await?;
            print_log(
                LogLevel::Info,
                &format!("Removed snapshot file at {snap_path:?}"),
            );
        }

        // Restart system services if requested
        restart_services().await;

        print_log(LogLevel::Fruitful, "Unapply operation complete.");
        notify(
            "Undoed changes.",
            "For a complete reset of your preferred domains, run `cutler reset`.",
        );

        Ok(())
    }
}
