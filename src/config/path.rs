// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;
use std::sync::OnceLock;

/// The configuration path decided for the current process.
pub static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Returns the path to the configuration file by checking several candidate locations.
pub fn get_config_path() -> PathBuf {
    if let Some(path) = CONFIG_PATH.get().cloned() {
        return path;
    }

    let home = dirs::home_dir();
    let xdg = dirs::config_dir();

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
    candidates
        .iter()
        .find(|f| f.try_exists().unwrap_or(false))
        .cloned()
        .unwrap_or_else(|| candidates[0].clone())
}
