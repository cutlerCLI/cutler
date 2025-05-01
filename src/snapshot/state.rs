use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

/// A single defaults‑setting change.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingState {
    pub domain: String,
    pub key: String,
    pub original_value: Option<String>,
    pub new_value: String,
}

/// One external command run.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExternalCommandState {
    pub run: String,
    pub sudo: bool,
}

/// The full snapshot on disk.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Snapshot {
    pub settings: Vec<SettingState>,
    pub commands: Vec<ExternalCommandState>,
    pub version: String,
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot {
            settings: Vec::new(),
            commands: Vec::new(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), anyhow::Error> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let txt = fs::read_to_string(path)?;
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
