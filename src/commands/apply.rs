use crate::{
    config::loader::load_config,
    defaults::{executor, flags},
    domains::collector,
    external::runner,
    snapshot::state::{SettingState, Snapshot},
    util::{
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};
use anyhow::Result;

/// Defines a job to be executed by the apply command.
#[derive(Debug)]
struct Job {
    domain: String,
    key: String,
    flag: String,
    value: String,
    action: &'static str,
    original: Option<String>,
    new_value: String,
}

pub async fn run(no_exec: bool, verbose: bool, dry_run: bool) -> Result<()> {
    let config_path = crate::config::loader::get_config_path();
    if !config_path.exists() {
        print_log(
            LogLevel::Info,
            &format!("Config not found at {:?}", config_path),
        );
        if confirm_action("Create new config?")? {
            super::init::run(verbose, false).await?;
            return Ok(());
        } else {
            anyhow::bail!("No config; aborting.");
        }
    }

    // parse + flatten domains
    let toml = load_config(&config_path).await?;
    let domains = collector::collect(&toml)?;

    // load the old snapshot (if any), otherwise create a new instance
    let snap_path = crate::snapshot::state::get_snapshot_path();
    let snap = if snap_path.exists() {
        Snapshot::load(&snap_path).await.unwrap_or_else(|e| {
            print_log(
                LogLevel::Warning,
                &format!("Bad snapshot: {}; starting new", e),
            );
            Snapshot::new()
        })
    } else {
        Snapshot::new()
    };

    // turn the old snapshot into a HashMap for a quick lookup
    let mut existing: std::collections::HashMap<_, _> = snap
        .settings
        .into_iter()
        .map(|s| ((s.domain.clone(), s.key.clone()), s))
        .collect();

    let mut jobs: Vec<Job> = Vec::new();

    for (dom, table) in domains.into_iter() {
        // if we need to insert the com.apple prefix, check once
        if collector::needs_prefix(&dom) {
            collector::check_exists(&format!("com.apple.{}", dom))?;
        }

        for (key, val) in table.into_iter() {
            let (eff_dom, eff_key) = collector::effective(&dom, &key);
            let desired = flags::normalize(&val);

            // read the current value from the system
            // then, check if changed
            let current = collector::read_current(&eff_dom, &eff_key).unwrap_or_default();
            let changed = current != desired;

            // grab the old snapshot entry if it exists
            let old_entry = existing.get(&(eff_dom.clone(), eff_key.clone())).cloned();

            if changed {
                existing.remove(&(eff_dom.clone(), eff_key.clone()));
                let original = old_entry.as_ref().and_then(|e| e.original_value.clone());

                // decide “Applying” vs “Updating”
                let action = if old_entry.is_some() {
                    "Updating"
                } else {
                    "Applying"
                };

                // turn TOML value into a -bool/-int/-string + stringified value
                let (flag, val_str) = flags::to_flag(&val)?;

                jobs.push(Job {
                    domain: eff_dom.clone(),
                    key: eff_key.clone(),
                    flag: flag.to_owned(),
                    value: val_str,
                    action,
                    original,
                    new_value: desired.clone(),
                });
            } else if verbose {
                print_log(
                    LogLevel::Info,
                    &format!("Skipping unchanged {}.{}", eff_dom, eff_key),
                );
            }
        }
    }

    // now execute writes concurrently with Tokio tasks
    let mut handles = Vec::with_capacity(jobs.len());
    for job in jobs.iter() {
        // clone for move into async task
        let domain = job.domain.clone();
        let key = job.key.clone();
        let flag = job.flag.clone();
        let value = job.value.clone();
        let action = job.action;

        handles.push(tokio::spawn(async move {
            let _ = executor::write(&domain, &key, &flag, &value, action, verbose, dry_run).await;
        }));
    }
    // await all write tasks
    for handle in handles {
        let _ = handle.await;
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
    new_snap.external = runner::extract(&toml);

    if !dry_run {
        new_snap.save(&snap_path).await?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Snapshot saved: {:?}", snap_path),
            );
        }
    } else {
        print_log(LogLevel::Info, "Dry-run: would save snapshot");
    }

    // exec external commands
    if !no_exec {
        let _ = runner::run_all(&toml, verbose, dry_run).await;
    }

    Ok(())
}
