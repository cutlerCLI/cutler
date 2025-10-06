// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf, sync::OnceLock};
use tokio::fs;

/// A single defaults‑setting change.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingState {
    pub domain: String,
    pub key: String,
    pub original_value: Option<String>,
}

/// The full snapshot on disk.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Snapshot {
    pub settings: Vec<SettingState>,
    pub exec_run_count: i32,
    pub version: String,
    #[serde(skip)]
    pub snapshot_path: PathBuf,
}

impl Snapshot {
    pub async fn is_loadable() -> bool {
        fs::try_exists(get_snapshot_path().unwrap_or_default())
            .await
            .unwrap_or_default()
    }

    pub fn new() -> Self {
        Snapshot {
            settings: Vec::new(),
            version: env!("CARGO_PKG_VERSION").into(),
            snapshot_path: get_snapshot_path().expect("Failed to get snapshot path"),
            exec_run_count: 0,
        }
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(dir) = self.snapshot_path.parent() {
            fs::create_dir_all(dir)
                .await
                .context("Failed to create snapshot directory")?;
        }
        let json = serde_json::to_string(self).context("Failed to serialize Snapshot to JSON")?;
        fs::write(&self.snapshot_path, json)
            .await
            .with_context(|| format!("Failed to write snapshot to {:?}", &self.snapshot_path))?;
        Ok(())
    }

    pub async fn load(path: &PathBuf) -> Result<Self> {
        let txt = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read snapshot file {:?}", path))?;
        let snap: Snapshot =
            serde_json::from_str(&txt).context("Failed to deserialize Snapshot from JSON")?;
        Ok(snap)
    }

    pub async fn delete(&self) -> Result<()> {
        fs::remove_file(&self.snapshot_path)
            .await
            .with_context(|| format!("Could not delete snapshot file {:?}.", &self.snapshot_path))
    }
}

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
