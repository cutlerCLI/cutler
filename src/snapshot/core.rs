// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use tokio::fs;

use crate::domains::convert::SerializablePrefValue;
use crate::snapshot::get_snapshot_path;

/// A single defaults‑setting change.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SettingState {
    pub domain: String,
    pub key: String,
    pub original_value: Option<SerializablePrefValue>,
}

/// Represents a snapshot.
///
/// This struct has also implemented I/O operations and functions for using across cutler's codebase,
/// in order to properly interact with the snapshot file without much hassle.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub settings: Vec<SettingState>,
    pub exec_run_count: i32,
    pub version: String,
    pub digest: String,
    #[serde(skip)]
    pub path: PathBuf,
}

impl Snapshot {
    /// Checks if the snapshot exists.
    /// This is a more tinified approach for regular `fs::try_exists()` calls as `get_snapshot_path()`
    /// returns a Result and could be cumbersome to implement everywhere in the codebase.
    pub async fn is_loadable() -> bool {
        if let Ok(snap_path) = get_snapshot_path().await {
            fs::try_exists(snap_path).await.unwrap_or_default()
        } else {
            false
        }
    }

    /// Creates a new snapshot.
    /// Note that the path field is decided by `get_snapshot_path()`.
    pub async fn new() -> Self {
        Self {
            settings: Vec::new(),
            version: env!("CARGO_PKG_VERSION").into(),
            path: get_snapshot_path()
                .await
                .expect("Failed to get snapshot path."),
            exec_run_count: 0,
            digest: String::new(),
        }
    }

    /// Saves the snapshot into the designated path for the instance.
    pub async fn save(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).await?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&self.path, json).await?;
        Ok(())
    }

    /// Loads the snapshot from the given path.
    /// If deserialization of the full Snapshot fails, try to deserialize only the `settings` field.
    pub async fn load(path: &PathBuf) -> Result<Self> {
        if fs::try_exists(path).await.unwrap_or_default() {
            let txt = fs::read_to_string(path).await?;
            let snap_result: Result<Self, _> = serde_json::from_str(&txt);

            match snap_result {
                Ok(mut snap) => {
                    snap.path = path.clone();
                    Ok(snap)
                }
                Err(e) => {
                    // Try to deserialize only the settings field if everything else fails.
                    #[derive(Deserialize)]
                    struct SettingsOnly {
                        settings: Vec<SettingState>,
                    }
                    let settings_only_result: Result<SettingsOnly, _> = serde_json::from_str(&txt);
                    match settings_only_result {
                        Ok(settings_only) => {
                            let mut snap = Self::new().await;
                            snap.settings = settings_only.settings;
                            snap.path = path.clone();
                            Ok(snap)
                        }
                        Err(_) => {
                            bail!("Failed to deserialize snapshot: {e}")
                        }
                    }
                }
            }
        } else {
            bail!("Invalid path, cannot load.")
        }
    }

    /// Deletes the snapshot.
    pub async fn delete(&self) -> Result<()> {
        fs::remove_file(&self.path)
            .await
            .with_context(|| format!("Could not delete snapshot file {:?}.", &self.path))
    }
}
