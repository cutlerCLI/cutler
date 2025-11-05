// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Result, bail};
use std::sync::OnceLock;
use std::{env, path::PathBuf};
use tokio::fs;

/// The configuration path decided for the current process.
pub static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Returns the path to the configuration file by checking several candidate locations.
pub async fn get_config_path() -> Result<PathBuf> {
    if let Some(path) = CONFIG_PATH.get().cloned() {
        return Ok(path);
    }

    let home = env::var_os("HOME");
    let xdg = env::var_os("XDG_CONFIG_HOME");

    let mut candidates = Vec::new();

    if let Some(ref home) = home {
        // $HOME/.config/cutler/config.toml
        candidates.push(
            PathBuf::from(home)
                .join(".config")
                .join("cutler")
                .join("config.toml"),
        );
        // $HOME/.config/cutler.toml
        candidates.push(PathBuf::from(home).join(".config").join("cutler.toml"));
    }

    if let Some(ref xdg) = xdg {
        // $XDG_CONFIG_HOME/cutler/config.toml
        candidates.push(PathBuf::from(xdg).join("cutler").join("config.toml"));
        // $XDG_CONFIG_HOME/cutler.toml
        candidates.push(PathBuf::from(xdg).join("cutler.toml"));
    }

    // Find the first existing candidate
    let chosen = if let Some(existing) = {
        let mut found = None;
        for candidate in &candidates {
            if fs::try_exists(candidate).await.unwrap_or(false) {
                found = Some(candidate.to_owned());
                break;
            }
        }
        found
    } {
        Some(existing)
    } else if !candidates.is_empty() {
        Some(candidates[0].clone())
    } else {
        None
    };

    if let Some(ref path) = chosen {
        CONFIG_PATH.set(path.clone()).ok();
        Ok(path.clone())
    } else {
        bail!("Could not load configuration since cannot be assigned.")
    }
}
