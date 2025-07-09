use crate::{
    commands::{BrewInstallCmd, Runnable},
    config::{
        loader::{get_config_path, load_config},
        remote::RemoteConfig,
    },
    domains::collector,
    exec::runner,
    snapshot::{
        get_snapshot_path,
        state::{SettingState, Snapshot},
    },
    util::{
        convert::{normalize, toml_to_prefvalue},
        globals::should_dry_run,
        io::{confirm_action, restart_system_services},
        logging::{GREEN, LogLevel, RESET, print_log},
    },
};
use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, preferences::Preferences};
use tokio::fs;
use toml::Value;

#[derive(Args, Debug)]
pub struct ApplyCmd {
    /// The URL to the remote config file.
    #[arg(short, long)]
    pub url: Option<String>,

    /// Skip executing external commands at the end.
    #[arg(long)]
    pub no_exec: bool,

    /// Risky: Disables check for domain existence before applying modification
    #[arg(long)]
    pub no_checks: bool,

    /// Invoke `cutler brew install` after applying defaults.
    #[arg(long)]
    pub with_brew: bool,
}

/// Represents an apply command job.
#[derive(Debug)]
struct Job {
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

        // remote download logic
        let config_path = get_config_path().await;

        if let Some(url) = &self.url {
            if fs::try_exists(&config_path).await.unwrap()
                && !confirm_action("Local config exists but a URL was still passed. Proceed?")
                    .unwrap()
            {
                bail!("Aborted apply: --url is passed despite local config.")
            }

            let remote_txt = RemoteConfig {
                url: url.clone(),
                autosync: true,
            }
            .fetch()
            .await?
            .as_table()
            .unwrap()
            .to_string();

            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&config_path, remote_txt).await?;

            print_log(
                LogLevel::Info,
                &format!("Remote config downloaded at path: {config_path:?}"),
            );
        }

        // parse + flatten domains
        let toml = load_config(true).await?;
        let domains = collector::collect(&toml)?;

        // load the old snapshot (if any), otherwise create a new instance
        let snap_path = get_snapshot_path();
        let snap = if fs::try_exists(&snap_path).await.unwrap() {
            Snapshot::load(&snap_path).await.unwrap_or_else(|e| {
                print_log(
                    LogLevel::Warning,
                    &format!("Bad snapshot: {e}; starting new"),
                );
                Snapshot::new()
            })
        } else {
            Snapshot::new()
        };

        // turn the old snapshot into a hashmap for a quick lookup
        let mut existing: std::collections::HashMap<_, _> = snap
            .settings
            .into_iter()
            .map(|s| ((s.domain.clone(), s.key.clone()), s))
            .collect();

        let mut jobs: Vec<Job> = Vec::new();

        for (dom, table) in domains.into_iter() {
            for (key, toml_value) in table.into_iter() {
                let (eff_dom, eff_key) = collector::effective(&dom, &key);

                if !self.no_checks {
                    collector::check_domain_exists(&eff_dom).await?;
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
                    let original = old_entry.as_ref().and_then(|e| e.original_value.clone());

                    // decide “applying” vs “updating”
                    let action = if old_entry.is_some() {
                        "Updating"
                    } else {
                        "Applying"
                    };

                    jobs.push(Job {
                        domain: eff_dom.clone(),
                        key: eff_key.clone(),
                        toml_value: toml_value.clone(),
                        action,
                        original,
                        new_value: desired.clone(),
                    });
                } else {
                    print_log(
                        LogLevel::Info,
                        &format!("Skipping unchanged {eff_dom} | {eff_key}"),
                    );
                }
            }
        }

        // use defaults-rs batch write API for all changed settings
        // collect jobs into a Vec<(Domain, String, PrefValue)>
        let mut batch: Vec<(Domain, String, defaults_rs::PrefValue)> = Vec::new();

        for job in &jobs {
            let domain_obj = if job.domain == "NSGlobalDomain" {
                Domain::Global
            } else {
                Domain::User(job.domain.clone())
            };

            if !dry_run {
                print_log(
                    LogLevel::Info,
                    &format!(
                        "{}{} {} | {} -> {}{}",
                        GREEN, job.action, job.domain, job.key, job.new_value, RESET
                    ),
                );
            }
            let pref_value = toml_to_prefvalue(&job.toml_value)?;
            batch.push((domain_obj, job.key.clone(), pref_value));
        }

        // perform batch write
        if !dry_run {
            match Preferences::write_batch(batch).await {
                Ok(_) => {
                    print_log(LogLevel::Info, "All preferences applied.");
                }
                Err(e) => {
                    print_log(LogLevel::Error, &format!("Batch write failed: {e}"));
                }
            }
        } else {
            for job in &jobs {
                print_log(
                    LogLevel::Dry,
                    &format!(
                        "Would {} setting '{}' for {}",
                        job.action, job.key, job.domain
                    ),
                );
            }
        }

        let mut new_snap = Snapshot::new();
        for ((_, _), mut old_entry) in existing.into_iter() {
            old_entry.new_value = old_entry.new_value.clone();
            new_snap.settings.push(old_entry);
        }
        // now append all the newly applied/updated settings
        for job in jobs {
            new_snap.settings.push(SettingState {
                domain: job.domain,
                key: job.key,
                original_value: job.original.clone(),
                new_value: job.new_value,
            });
        }
        new_snap.external = runner::extract_all_cmds(&toml);

        if !dry_run {
            new_snap.save(&snap_path).await?;
            print_log(LogLevel::Info, &format!("Snapshot saved: {snap_path:?}"));
        } else {
            print_log(LogLevel::Dry, "Would save snapshot");
        }

        // run brew
        if self.with_brew {
            BrewInstallCmd.run().await?;
        }

        // exec external commands
        if !self.no_exec {
            runner::run_all(&toml).await?;
        }

        // restart system services if requested
        restart_system_services().await?;

        Ok(())
    }
}
