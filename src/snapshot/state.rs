// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use tokio::fs;

/// A single defaults‑setting change.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingState {
    pub domain: String,
    pub key: String,
    pub original_value: Option<String>,
}

/// One external command run.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalCommandState {
    pub name: String,
    pub run: String,
    pub sudo: bool,
    pub ensure_first: bool,
    pub flag: bool,
    pub required: Vec<String>,
}

/// The full snapshot on disk.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Snapshot {
    pub settings: Vec<SettingState>,
    pub external: Vec<ExternalCommandState>,
    pub version: String,
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot {
            settings: Vec::new(),
            external: Vec::new(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    pub async fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).await?;
        }
        let json = serde_json::to_string(self)?;
        fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load(path: &PathBuf) -> Result<Self> {
        let txt = fs::read_to_string(path).await?;
        let snap: Snapshot = serde_json::from_str(&txt)?;
        Ok(snap)
    }
}

/// Where on disk the snapshot lives (`~/.cutler_snapshot`).
pub fn get_snapshot_path() -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home).join(".cutler_snapshot")
    } else {
        PathBuf::from(".cutler_snapshot")
    }
}
