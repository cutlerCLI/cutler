use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{env, fs};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingState {
    pub domain: String,
    pub key: String,
    pub original_value: Option<String>, // None if setting didn't exist before
    pub new_value: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ExternalCommandState {
    pub cmd: String,
    pub args: Vec<String>,
    pub sudo: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Snapshot {
    pub settings: Vec<SettingState>,
    pub external_commands: Vec<ExternalCommandState>,
    pub version: String, // Snapshot format version for future compatibility
}

/// Returns the path where the snapshot is stored.
pub fn get_snapshot_path() -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home).join(".cutler_snapshot")
    } else {
        PathBuf::from(".cutler_snapshot")
    }
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot {
            settings: Vec::new(),
            external_commands: Vec::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        match serde_json::from_str(&content) {
            Ok(snapshot) => Ok(snapshot),
            Err(_) => Err(format!(
                "Failed to parse snapshot. You can:\n\n1. Delete the file manually: rm {}\n2. Use 'cutler reset' to reset settings to defaults\n\nNote: Without a valid snapshot, cutler cannot restore previous values.",
                path.display()
            )
            .into()),
        }
    }
}
