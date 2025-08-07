use std::{env, path::PathBuf};

use tokio::fs;

/// Returns the path to the configuration file by checking several candidate locations.
pub async fn get_config_path() -> PathBuf {
    let mut candidates = Vec::new();

    // decide candidates in order
    let home = env::var_os("HOME");

    if let Some(ref home) = home {
        let candidate = PathBuf::from(home)
            .join(".config")
            .join("cutler")
            .join("config.toml");
        candidates.push(candidate);

        let candidate2 = PathBuf::from(home).join(".config").join("cutler.toml");
        candidates.push(candidate2);
    }

    candidates.push(PathBuf::from("cutler.toml"));

    // return the first candidate that exists
    for candidate in &candidates {
        if fs::try_exists(candidate).await.unwrap_or(false) {
            return candidate.to_owned();
        }
    }

    // if none exist, always return $HOME/.config/cutler/config.toml if HOME is set
    // else fallback to .config/cutler/config.toml in current directory
    if let Some(home) = home {
        PathBuf::from(home)
            .join(".config")
            .join("cutler")
            .join("config.toml")
    } else {
        PathBuf::from(".config")
            .join("cutler")
            .join("config.toml")
    }
}
