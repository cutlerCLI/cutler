// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use tokio::fs;

use crate::snapshot::get_snapshot_path;

/// A single defaults‑setting change.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingState {
    pub domain: String,
    pub key: String,
    pub original_value: Option<String>,
}

/// Represents a snapshot.
///
/// This struct has also implemented I/O operations and functions for using across cutler's codebase,
/// in order to properly interact with the snapshot file without much hassle.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Snapshot {
    pub settings: Vec<SettingState>,
    pub exec_run_count: i32,
    pub version: String,
    #[serde(skip)]
    pub path: PathBuf,
}

impl Snapshot {
    /// Checks if the snapshot exists.
    /// This is a more tinified approach for regular fs::try_exists() calls as get_snapshot_path() returns a Result
    /// and could be cumbersome to implement everywhere in the codebase.
    pub async fn is_loadable() -> bool {
        fs::try_exists(get_snapshot_path().unwrap_or_default())
            .await
            .unwrap_or_default()
    }

    /// Creates a new snapshot.
    /// NOTE: Snapshot.path is decided by get_snapshot_path().
    pub fn new() -> Self {
        Snapshot {
            settings: Vec::new(),
            version: env!("CARGO_PKG_VERSION").into(),
            path: get_snapshot_path().expect("Failed to get snapshot path"),
            exec_run_count: 0,
        }
    }

    /// Saves the snapshot into the designated path for the instance.
    pub async fn save(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)
                .await
                .context("Failed to create snapshot directory")?;
        }
        let json = serde_json::to_string(self).context("Failed to serialize Snapshot to JSON")?;
        fs::write(&self.path, json)
            .await
            .with_context(|| format!("Failed to write snapshot to {:?}", &self.path))?;
        Ok(())
    }

    /// Loads the snapshot from the given path.
    pub async fn load(path: &PathBuf) -> Result<Self> {
        let txt = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read snapshot file {:?}", path))?;
        let mut snap: Snapshot =
            serde_json::from_str(&txt).context("Failed to deserialize Snapshot from JSON")?;

        snap.path = path.clone();
        Ok(snap)
    }

    /// Deletes the snapshot.
    pub async fn delete(&self) -> Result<()> {
        fs::remove_file(&self.path)
            .await
            .with_context(|| format!("Could not delete snapshot file {:?}.", &self.path))
    }
}
