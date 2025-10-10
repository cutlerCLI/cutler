use anyhow::{Context, Result};
use std::{path::PathBuf, sync::OnceLock};
use tokio::fs;

use crate::config::path::get_config_path;

/// The static snapshot path to use throughout each command run.
/// This is to make sure that accidental variable changes don't alter the snapshot being written.
static SNAP_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Returns the path to the snapshot file ($HOME/.cutler_snapshot).
/// If for some reason the home directory cannot be detected, this function will return None.
/// It also initializes the path once, meaning that all future calls from the first one will
/// return the same path despite of snapshot changes.
pub async fn get_snapshot_path() -> Result<PathBuf> {
    if let Some(cached) = SNAP_PATH.get().cloned() {
        return Ok(cached);
    }

    let config_parent = get_config_path()
        .await?
        .parent()
        .context("Could not determine config parent directory")?
        .to_path_buf();

    let old_home = dirs::home_dir().context("Could not determine home directory")?;
    let old_path = old_home.join(".cutler_snapshot");
    let new_path = config_parent.join("snapshot.json");

    // If the old snapshot exists, move it to the new path
    if old_path.exists() {
        // If the new path already exists, remove it before moving
        if new_path.exists() {
            fs::remove_file(&new_path)
                .await
                .with_context(|| format!("Failed to remove existing snapshot at {:?}", new_path))?;
        }
        fs::rename(&old_path, &new_path).await.with_context(|| {
            format!(
                "Failed to move snapshot from {:?} to {:?}",
                old_path, new_path
            )
        })?;
    }

    SNAP_PATH.set(new_path.clone()).ok();
    Ok(new_path)
}
