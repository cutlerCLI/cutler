// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, PrefValue, Preferences};

use crate::{
    cli::atomic::should_dry_run,
    commands::{ResetCmd, Runnable},
    config::Config,
    domains::convert::serializable_to_prefvalue,
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
    fn needs_sudo(&self) -> bool {
        false
    }

    async fn run(&self, config: &Config) -> Result<()> {
        if !Snapshot::is_loadable().await {
            log_warn!("No snapshot found to revert.");

            if confirm("Reset all System Settings instead?") {
                return ResetCmd.run(config).await;
            }
            bail!("Abort operation.")
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

        if snapshot.digest != get_digest(config.path.clone())? {
            log_warn!("Config has been modified since last application.",);
            log_warn!("Please note that only the applied modifications will be unapplied.",);
        }

        // prepare undo operations, grouping by domain for efficiency
        let mut restore_jobs: Vec<(Domain, String, PrefValue)> = Vec::new();
        let mut delete_jobs: Vec<(Domain, String)> = Vec::new();

        // reverse order to undo in correct sequence
        for s in snapshot.settings.clone().into_iter().rev() {
            let domain_obj = if s.domain == "NSGlobalDomain" {
                Domain::Global
            } else {
                Domain::User(s.domain.clone())
            };

            if let Some(orig) = s.original_value {
                let pref_value = serializable_to_prefvalue(&orig);

                restore_jobs.push((domain_obj, s.key, pref_value));
            } else {
                delete_jobs.push((domain_obj, s.key));
            }
        }

        // in dry-run mode, just print what would be done
        if dry_run {
            for (domain, key, original_value) in restore_jobs {
                log_dry!("Would restore: {domain} | {key} -> {original_value}",);
            }
            for (domain, key) in &delete_jobs {
                log_dry!("Would delete setting: {domain} | {key}",);
            }

            log_dry!("Would delete snapshot at path: {:?}", snapshot.path);
        } else {
            let mut settings_modified_count = 0;

            if restore_jobs.is_empty() {
                for (domain, key, value) in restore_jobs {
                    log_info!("Restoring: {domain} | {key} -> {value}",);

                    if let Err(e) = Preferences::write(domain.clone(), &key, value.clone()) {
                        log_err!("Restore failed: {e}");
                    } else {
                        settings_modified_count += 1;
                    }
                }
            }

            if !delete_jobs.is_empty() {
                for (domain, key) in delete_jobs {
                    log_info!("Deleting: {domain} | {key}");

                    if let Err(e) = Preferences::delete(domain.clone(), &key) {
                        log_err!("Delete failed: {e}");
                    } else {
                        settings_modified_count += 1;
                    }
                }
            }

            if snapshot.exec_run_count > 0 {
                log_warn!(
                    "{} commands were executed previously; revert them manually.",
                    snapshot.exec_run_count
                );
            }

            if settings_modified_count > 0 {
                log_info!("Modified {settings_modified_count} settings; restarting services.");
                restart_services().await;
            }

            snapshot.delete().await?;
            log_cute!("Unapply operation complete.");
        }

        Ok(())
    }
}
