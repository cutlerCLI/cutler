use anyhow::{Context, Result};
use std::{path::PathBuf, sync::OnceLock};

/// The static snapshot path to use throughout each command run.
/// This is to make sure that accidental variable changes don't alter the snapshot being written.
static SNAP_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Where on disk the snapshot lives (`~/.cutler_snapshot`).
pub fn get_snapshot_path() -> Result<PathBuf> {
    if let Some(cached) = SNAP_PATH.get().cloned() {
        return Ok(cached);
    }

    let home = dirs::home_dir().context("Could not determine home directory")?;
    let path = home.join(".cutler_snapshot");
    SNAP_PATH.set(path.clone()).ok();
    Ok(path)
}
