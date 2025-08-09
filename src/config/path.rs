use std::{env, path::PathBuf};

use tokio::fs;

/// Returns the path to the configuration file by checking several candidate locations.
pub async fn get_config_path() -> PathBuf {
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
    for candidate in &candidates {
        if fs::try_exists(candidate).await.unwrap() {
            return candidate.to_owned();
        }
    }

    // if none exist, always return the HOME-based config location
    if let Some(home) = home {
        PathBuf::from(home)
            .join(".config")
            .join("cutler")
            .join("config.toml")
    } else {
        PathBuf::from("~")
            .join(".config")
            .join("cutler")
            .join("config.toml")
    }
}
