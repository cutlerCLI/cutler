use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, preferences::Preferences};
use std::collections::HashMap;
use tokio::fs;

use crate::{
    commands::Runnable,
    defaults::convert::toml_to_prefvalue,
    snapshot::state::{Snapshot, get_snapshot_path},
    util::{
        globals::should_dry_run,
        io::restart_system_services_if_needed,
        logging::{LogLevel, print_log},
    },
};

/// Helper: turn string to TOML value
fn string_to_toml_value(s: &str) -> toml::Value {
    // try bool, int, float, fallback to string
    if s == "true" {
        toml::Value::Boolean(true)
    } else if s == "false" {
        toml::Value::Boolean(false)
    } else if let Ok(i) = s.parse::<i64>() {
        toml::Value::Integer(i)
    } else if let Ok(f) = s.parse::<f64>() {
        toml::Value::Float(f)
    } else {
        toml::Value::String(s.to_string())
    }
}

#[derive(Args, Debug)]
pub struct UnapplyCmd;

#[async_trait]
impl Runnable for UnapplyCmd {
    async fn run(&self) -> Result<()> {
        let snap_path = get_snapshot_path();

        if !snap_path.exists() {
            bail!(
                "No snapshot found. Please run `cutler apply` first before unapplying.\n\
                As a fallback, you can use `cutler reset` to reset settings to defaults."
            );
        }

        let dry_run = should_dry_run();

        // load snapshot from disk
        let snapshot = Snapshot::load(&snap_path)
            .await
            .context(format!("Failed to load snapshot from {:?}", snap_path))?;

        // prepare undo operations, grouping by domain for efficiency
        let mut batch_restores: HashMap<Domain, Vec<(String, defaults_rs::PrefValue)>> =
            HashMap::new();
        let mut batch_deletes: HashMap<Domain, Vec<String>> = HashMap::new();

        // reverse order to undo in correct sequence
        for s in snapshot.settings.into_iter().rev() {
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
                for (key, _) in restores {
                    let domain_str = match domain {
                        Domain::Global => "NSGlobalDomain",
                        Domain::User(s) => s,
                        _ => unreachable!(),
                    };
                    print_log(
                        LogLevel::Dry,
                        &format!("Would restore setting '{}' for {}", key, domain_str),
                    );
                }
            }
            for (domain, deletes) in &batch_deletes {
                for key in deletes {
                    let domain_str = match domain {
                        Domain::Global => "NSGlobalDomain",
                        Domain::User(s) => s,
                        _ => unreachable!(),
                    };
                    print_log(
                        LogLevel::Dry,
                        &format!("Would remove setting '{}' for {}", key, domain_str),
                    );
                }
            }
        } else {
            // perform batch restores
            if !batch_restores.is_empty() {
                let mut batch_vec = Vec::new();
                for (domain, entries) in batch_restores {
                    for (key, value) in entries {
                        batch_vec.push((domain.clone(), key, value));
                    }
                }
                match Preferences::write_batch(batch_vec).await {
                    Ok(_) => {
                        print_log(LogLevel::Success, "All settings restored (batch write).");
                    }
                    Err(e) => {
                        print_log(LogLevel::Error, &format!("Batch restore failed: {e}"));
                    }
                }
            }

            // perform batch deletes
            if !batch_deletes.is_empty() {
                let mut delete_vec = Vec::new();
                for (domain, keys) in batch_deletes {
                    for key in keys {
                        delete_vec.push((domain.clone(), Some(key)));
                    }
                }
                match Preferences::delete_batch(delete_vec).await {
                    Ok(_) => {
                        print_log(LogLevel::Success, "All settings removed (batch delete).");
                    }
                    Err(e) => {
                        print_log(LogLevel::Error, &format!("Batch delete failed: {e}"));
                    }
                }
            }
        }

        // warn about external commands (not automatically reverted)
        if !snapshot.external.is_empty() {
            print_log(
                LogLevel::Warning,
                "External commands were executed previously; please revert them manually if needed.",
            );
        }

        // delete the snapshot file
        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would remove snapshot file at {:?}", snap_path),
            );
        } else {
            fs::remove_file(&snap_path)
                .await
                .context(format!("Failed to remove snapshot file at {:?}", snap_path))?;
            print_log(
                LogLevel::Success,
                &format!("Removed snapshot file at {:?}", snap_path),
            );
        }

        // Restart system services if requested
        restart_system_services_if_needed().await?;

        Ok(())
    }
}
