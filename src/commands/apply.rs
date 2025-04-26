use crate::{
    config::loader::load_config,
    defaults::{executor, flags},
    domains::collector,
    external::runner,
    snapshot::state::{SettingState, Snapshot},
    util::io::confirm_action,
    util::logging::{LogLevel, print_log},
};
use anyhow::Result;
use rayon::prelude::*;

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

pub fn run(no_exec: bool, verbose: bool, dry_run: bool) -> Result<()> {
    let config_path = crate::config::loader::get_config_path();
    if !config_path.exists() {
        print_log(
            LogLevel::Info,
            &format!("Config not found at {:?}", config_path),
        );
        if confirm_action("Create new config?")? {
            super::init::run(verbose, false)?;
            print_log(
                LogLevel::Info,
                "Config created; edit it and then `cutler apply`.",
            );
            return Ok(());
        } else {
            anyhow::bail!("No config; aborting.");
        }
    }

    // parse + flatten domains
    let toml = load_config(&config_path)?;
    let domains = collector::collect(&toml)?;

    // load old snapshot (if any), otherwise create a new instance
    let snap_path = crate::snapshot::state::get_snapshot_path();
    let snap = if snap_path.exists() {
        Snapshot::load(&snap_path).unwrap_or_else(|e| {
            print_log(
                LogLevel::Warning,
                &format!("Bad snapshot: {}; starting new", e),
            );
            Snapshot::new()
        })
    } else {
        Snapshot::new()
    };

    // turn old snapshot into a HashMap for quick lookup
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

            // see if there's an existing entry
            let old_entry = existing.remove(&(eff_dom.clone(), eff_key.clone()));

            // did the value actually change?
            let changed = old_entry
                .as_ref()
                .map(|e| e.new_value != desired)
                .unwrap_or(true);

            if changed {
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
                    original: old_entry.as_ref().and_then(|e| e.original_value.clone()),
                    new_value: desired.clone(),
                });
            } else if verbose {
                print_log(
                    LogLevel::Info,
                    &format!("Skipping unchanged {}.{}", eff_dom, eff_key),
                );
                // put it back
                existing.insert((eff_dom.clone(), eff_key.clone()), old_entry.unwrap());
            }
        }
    }

    // now execute writes in parallel
    jobs.par_iter().for_each(|job| {
        // each executor::write now has your domain‐locking built in
        let _ = executor::write(
            &job.domain,
            &job.key,
            &job.flag,
            &job.value,
            job.action,
            verbose,
            dry_run,
        );
    });

    let mut new_snap = Snapshot::new();
    for ((_, _), mut old_entry) in existing.into_iter() {
        old_entry.new_value = old_entry.new_value.clone();
        new_snap.settings.push(old_entry);
    }
    // now append all of the newly applied/updated settings
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
        new_snap.save(&snap_path)?;
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
        let _ = runner::run_all(&toml, verbose, dry_run);
    }

    Ok(())
}
