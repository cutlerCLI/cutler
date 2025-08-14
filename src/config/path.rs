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

    let mut candidates = Vec::new();

    // decide candidates in order
    let home = env::var_os("HOME");

    if let Some(ref home) = home {
        let candidate1 = PathBuf::from(home)
            .join(".config")
            .join("cutler")
            .join("config.toml");
        candidates.push(candidate1);

        let candidate2 = PathBuf::from(home).join(".config").join("cutler.toml");
        candidates.push(candidate2);
    }

    // return the first candidate that exists
    let chosen = if let Some(existing) = {
        let mut found = None;
        for candidate in &candidates {
            if fs::try_exists(candidate).await.unwrap() {
                found = Some(candidate.to_owned());
                break;
            }
        }
        found
    } {
        existing
    } else if let Some(home) = home {
        PathBuf::from(home)
            .join(".config")
            .join("cutler")
            .join("config.toml")
    } else {
        PathBuf::from("~")
            .join(".config")
            .join("cutler")
            .join("config.toml")
    };

    CONFIG_PATH.set(chosen.clone()).ok();
    chosen
}
