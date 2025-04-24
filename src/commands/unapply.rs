use anyhow::{Context, Result, bail};
use std::fs;

use crate::{
    defaults::{executor, flags},
    snapshot::state::{Snapshot, get_snapshot_path},
    util::logging::{LogLevel, print_log},
};

/// Unapply settings using the stored snapshot
pub fn run(verbose: bool, dry_run: bool) -> Result<()> {
    let snap_path = get_snapshot_path();

    if !snap_path.exists() {
        bail!(
            "No snapshot found. Please run `cutler apply` first before unapplying.\n\
            As a fallback, you can use `cutler reset` to reset settings to defaults."
        );
    }

    // load snapshot from disk
    let snapshot = Snapshot::load(&snap_path)
        .context(format!("Failed to load snapshot from {:?}", snap_path))?;

    // walk settings in reverse order
    for setting in snapshot.settings.iter().rev() {
        match &setting.original_value {
            Some(orig) => {
                // Restore original value
                let (flag, value_str) = flags::from_flag(orig)
                    .context("Could not determine flag for original value")?;
                executor::write(
                    &setting.domain,
                    &setting.key,
                    flag,
                    &value_str,
                    "Restoring",
                    verbose,
                    dry_run,
                )?;
                if verbose {
                    print_log(
                        LogLevel::Success,
                        &format!(
                            "Restored {}.{} to original value {}",
                            setting.domain, setting.key, orig
                        ),
                    );
                }
            }
            None => {
                // Key did not exist before—delete it now
                executor::delete(&setting.domain, &setting.key, "Removing", verbose, dry_run)?;
                if verbose {
                    print_log(
                        LogLevel::Success,
                        &format!(
                            "Removed {}.{} (didn't exist before cutler)",
                            setting.domain, setting.key
                        ),
                    );
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
            LogLevel::Info,
            &format!("Dry-run: Would remove snapshot file at {:?}", snap_path),
        );
    } else {
        fs::remove_file(&snap_path)
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
