use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use tokio::fs;

/// A single defaults‑setting change.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingState {
    /// The domain for the preference.
    pub domain: String,
    /// The key to change.
    pub key: String,
    /// The original value for the domain-key pair.
    pub original_value: Option<String>,
    /// The new value for the domain-key pair.
    pub new_value: String,
}

/// One external command run.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExternalCommandState {
    /// The actual command to run.
    pub run: String,
    /// Annotated sudo (optional if separated commands need sudo).
    pub sudo: bool,
    /// Run before any other commands (in serial order).
    pub ensure_first: bool,
    /// Only run if `--exec-all` is passed in with `cutler apply`.
    pub flag_only: bool,
    /// Required apps in $PATH for the commmand.
    pub required: Vec<String>,
}

/// The full snapshot on disk.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Snapshot {
    /// Preferences which have been applied.
    pub settings: Vec<SettingState>,
    /// External commands which have been run.
    pub external: Vec<ExternalCommandState>,
    /// The version of cutler which created the snapshot.
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

    pub async fn save(&self, path: &PathBuf) -> Result<(), anyhow::Error> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).await?;
        }
        let json = serde_json::to_string(self)?;
        fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load(path: &PathBuf) -> Result<Self, anyhow::Error> {
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
