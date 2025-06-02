use anyhow::{Context, Result, bail};
use tokio::fs;

use crate::{
    defaults::{executor, from_flag},
    snapshot::state::{Snapshot, get_snapshot_path},
    util::logging::{LogLevel, print_log},
};

/// Defines an undo operation to be executed by the unapply command.
#[derive(Clone)]
enum Undo {
    Restore {
        domain: String,
        key: String,
        orig: String,
    },
    Delete {
        domain: String,
        key: String,
    },
}

pub async fn run(verbose: bool, dry_run: bool) -> Result<()> {
    let snap_path = get_snapshot_path();

    if !snap_path.exists() {
        bail!(
            "No snapshot found. Please run `cutler apply` first before unapplying.\n\
            As a fallback, you can use `cutler reset` to reset settings to defaults."
        );
    }

    // load snapshot from disk
    let snapshot = Snapshot::load(&snap_path)
        .await
        .context(format!("Failed to load snapshot from {:?}", snap_path))?;

    // list which values to restore / delete
    let undoes: Vec<Undo> = snapshot
        .settings
        .into_iter()
        .rev()
        .map(|s| {
            if let Some(o) = s.original_value.clone() {
                Undo::Restore {
                    domain: s.domain,
                    key: s.key,
                    orig: o,
                }
            } else {
                Undo::Delete {
                    domain: s.domain,
                    key: s.key,
                }
            }
        })
        .collect();

    // run undo concurrently
    let mut handles = Vec::new();
    for u in undoes {
        handles.push(tokio::spawn(async move {
            match u {
                Undo::Restore { domain, key, orig } => {
                    let (flag, val_str) = from_flag(&orig).unwrap();
                    let _ = executor::write(
                        &domain,
                        &key,
                        flag,
                        &val_str,
                        "Restoring",
                        verbose,
                        dry_run,
                    )
                    .await;
                }
                Undo::Delete { domain, key } => {
                    let _ = executor::delete(&domain, &key, "Removing", verbose, dry_run).await;
                }
            }
        }));
    }
    for handle in handles {
        let _ = handle.await;
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
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Removed snapshot file at {:?}", snap_path),
            );
        }
    }

    Ok(())
}
