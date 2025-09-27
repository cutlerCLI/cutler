// SPDX-License-Identifier: MIT

use std::sync::OnceLock;
use std::{env, path::PathBuf};
use tokio::fs;

/// The configuration path decided for the current process.
static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Returns the path to the configuration file by checking several candidate locations.
pub async fn get_config_path() -> PathBuf {
    if let Some(path) = CONFIG_PATH.get() {
        return path.clone();
    }

    let home = env::var_os("HOME");
    let xdg = env::var_os("XDG_CONFIG_HOME");

    let mut candidates = Vec::new();

    // $HOME/.config/cutler/config.toml
    if let Some(ref home) = home {
        candidates.push(
            PathBuf::from(home)
                .join(".config")
                .join("cutler")
                .join("config.toml"),
        );
    }

    // $HOME/.config/cutler.toml
    if let Some(ref home) = home {
        candidates.push(PathBuf::from(home).join(".config").join("cutler.toml"));
    }

    // $XDG_CONFIG_HOME/cutler/config.toml
    if let Some(ref xdg) = xdg {
        candidates.push(PathBuf::from(xdg).join("cutler").join("config.toml"));
    }

    // $XDG_CONFIG_HOME/cutler.toml
    if let Some(ref xdg) = xdg {
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
        existing
    } else if !candidates.is_empty() {
        candidates[0].clone()
    } else {
        // fallback if no $HOME or $XDG_CONFIG_HOME is set
        let mut fallback = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        fallback = fallback.join(".config").join("cutler").join("config.toml");
        fallback
    };

    CONFIG_PATH.set(chosen.clone()).ok();
    chosen
}
