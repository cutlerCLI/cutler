// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    cli::atomic::should_dry_run,
    commands::{BrewInstallCmd, Runnable},
    config::{core::Config, path::get_config_path, remote::RemoteConfigManager},
    domains::{
        collector,
        convert::{normalize, toml_to_prefvalue},
    },
    exec::core::{self, ExecMode},
    log_cute, log_dry, log_err, log_info, log_warn,
    snapshot::{
        core::{SettingState, Snapshot},
        get_snapshot_path,
    },
    util::{
        io::{confirm, restart_services},
        sha::get_digest,
    },
};
use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, PrefValue, Preferences};
use toml::Value;

#[derive(Args, Debug)]
pub struct ApplyCmd {
    /// The URL to the remote config file.
    #[arg(short, long)]
    url: Option<String>,

    /// Skip executing external commands.
    #[arg(long, conflicts_with_all = &["all_cmd", "flagged_cmd"])]
    no_cmd: bool,

    /// Execute all external commands (even flagged ones).
    #[arg(short, long, conflicts_with_all = &["no_cmd", "flagged_cmd"])]
    all_cmd: bool,

    /// Execute flagged external commands only.
    #[arg(short, long, conflicts_with_all = &["all_cmd", "no_cmd"])]
    flagged_cmd: bool,

    /// Risky: Disables check for domain existence before applying modification.
    #[arg(long)]
    no_domain_check: bool,

    /// Invoke `brew install` after applying defaults.
    #[arg(short, long)]
    brew: bool,
}

/// Represents a preference modification job.
#[derive(Debug)]
struct PreferenceJob {
    domain: String,
    key: String,
    toml_value: Value,
    action: &'static str,
    original: Option<String>,
    new_value: String,
}

#[async_trait]
impl Runnable for ApplyCmd {
    async fn run(&self) -> Result<()> {
        let dry_run = should_dry_run();
        let config = Config::load(true).await?;

        // remote download logic
        if let Some(url) = &self.url {
            if Config::is_loadable().await
                && !confirm("Local config exists but a URL was still passed. Proceed?")
            {
                bail!("Aborted apply: --url is passed despite local config.")
            }

            let remote_mgr = RemoteConfigManager::new(url.to_owned());
            remote_mgr.fetch().await?;
            remote_mgr.save().await?;

            log_info!(
                "Remote config downloaded at path: {:?}",
                get_config_path().await
            );
        }

        // parse + flatten domains
        let digest = get_digest(config.path.clone())?;
        let domains = collector::collect(&config)?;

        // load the old snapshot (if any), otherwise create a new instance
        let snap_path = get_snapshot_path().await?;
        let mut is_bad_snap: bool = false;
        let snap = if Snapshot::is_loadable().await {
            match Snapshot::load(&snap_path).await {
                Ok(snap) => snap,
                Err(e) => {
                    log_warn!(
                        "Bad snapshot: {e}; starting new. Note that when unapplying, all your settings will reset to factory defaults."
                    );
                    is_bad_snap = true;
                    Snapshot::new().await
                }
            }
        } else {
            Snapshot::new().await
        };

        // turn the old snapshot into a hashmap for a quick lookup
        let mut existing: std::collections::HashMap<_, _> = snap
            .settings
            .into_iter()
            .map(|s| ((s.domain.clone(), s.key.clone()), s))
            .collect();

        let mut jobs: Vec<PreferenceJob> = Vec::new();

        let domains_list = Preferences::list_domains().await?;
        for (dom, table) in domains.into_iter() {
            for (key, toml_value) in table.into_iter() {
                let (eff_dom, eff_key) = collector::effective(&dom, &key);

                if !self.no_domain_check
                    && eff_dom != "NSGlobalDomain"
                    && !domains_list.contains(&eff_dom.to_owned())
                {
                    bail!("Domain \"{}\" does not exist!", eff_dom)
                }

                // read the current value from the system
                // then, check if changed
                // TODO: could use read_batch from defaults-rs here
                let current = collector::read_current(&eff_dom, &eff_key)
                    .await
                    .unwrap_or_default();
                let desired = normalize(&toml_value);
                let changed = current != desired;

                // grab the old snapshot entry if it exists
                let old_entry = existing.get(&(eff_dom.clone(), eff_key.clone())).cloned();

                if changed {
                    existing.remove(&(eff_dom.clone(), eff_key.clone()));

                    // Preserve existing non-null original; otherwise, for brand new keys, capture original from system
                    let original = if let Some(e) = &old_entry {
                        e.original_value.clone()
                    } else if current.is_empty() {
                        None
                    } else {
                        Some(current.clone())
                    };

                    // decide “applying” vs “updating”
                    let action = if old_entry.is_some() {
                        "Updating"
                    } else {
                        "Applying"
                    };

                    jobs.push(PreferenceJob {
                        domain: eff_dom.clone(),
                        key: eff_key.clone(),
                        toml_value: toml_value.clone(),
                        action,
                        original: if is_bad_snap { None } else { original },
                        new_value: desired.clone(),
                    });
                } else {
                    log_info!("Skipping unchanged {eff_dom} | {eff_key}",);
                }
            }
        }

        // use defaults-rs batch write API for all changed settings
        // collect jobs into a Vec<(Domain, String, PrefValue)>
        let mut batch: Vec<(Domain, String, PrefValue)> = Vec::new();

        for job in &jobs {
            let domain_obj = if job.domain == "NSGlobalDomain" {
                Domain::Global
            } else {
                Domain::User(job.domain.clone())
            };

            if !dry_run {
                log_info!(
                    "{} {} | {} -> {} {}",
                    job.action,
                    job.domain,
                    job.key,
                    job.new_value,
                    if job.original.is_some() {
                        format!("[Restorable to {}]", job.original.clone().unwrap())
                    } else {
                        "".to_string()
                    }
                );
            }
            let pref_value = toml_to_prefvalue(&job.toml_value)?;
            batch.push((domain_obj, job.key.clone(), pref_value));
        }

        // perform batch write
        if !dry_run {
            match Preferences::write_batch(batch).await {
                Ok(_) => {
                    log_info!("All preferences applied.");
                }
                Err(e) => {
                    log_err!("Batch write failed: {e}");
                }
            }

            // restart system services if requested
            restart_services().await;
        } else {
            for job in &jobs {
                log_dry!(
                    "Would {} setting '{}' for {}",
                    job.action,
                    job.key,
                    job.domain
                );
            }
        }

        let mut new_snap = Snapshot::new().await;
        for ((_, _), old_entry) in existing.into_iter() {
            new_snap.settings.push(old_entry);
        }

        // now append all the newly applied/updated settings
        for job in jobs {
            new_snap.settings.push(SettingState {
                domain: job.domain,
                key: job.key,
                original_value: job.original.clone(),
            });
        }

        // save config digest to snapshot
        new_snap.digest = digest;

        if !dry_run {
            new_snap.save().await?;
            log_info!("Logged system preferences change in snapshot.",);
        } else {
            log_dry!("Would save snapshot with system preferences.",);
        }

        // run brew
        if self.brew {
            BrewInstallCmd.run().await?;
        }

        // exec external commands
        if !self.no_cmd {
            let mode = if self.all_cmd {
                ExecMode::All
            } else if self.flagged_cmd {
                ExecMode::Flagged
            } else {
                ExecMode::Regular
            };

            let exec_run_count = core::run_all(config, mode).await?;

            if !dry_run {
                if exec_run_count > 0 {
                    new_snap.exec_run_count = exec_run_count;
                    new_snap.save().await?;

                    log_info!("Logged command execution in snapshot.");
                }
            } else {
                log_dry!("Would save snapshot with external command execution.",);
            }
        }

        log_cute!("Apply operation complete.");

        Ok(())
    }
}
