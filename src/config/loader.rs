use std::{
    env, fs,
    path::{Path, PathBuf},
};

use toml::Value;

/// Returns the path to the configuration file by checking several candidate locations.
pub fn get_config_path() -> PathBuf {
    let mut candidates = Vec::new();

    // decide candidates in order
    if let Some(xdg_config) = env::var_os("XDG_CONFIG_HOME") {
        let candidate = PathBuf::from(xdg_config).join("cutler").join("config.toml");
        candidates.push(candidate);
    }

    if let Some(home) = env::var_os("HOME") {
        let candidate = PathBuf::from(&home)
            .join(".config")
            .join("cutler")
            .join("config.toml");
        candidates.push(candidate);

        let candidate2 = PathBuf::from(home).join(".config").join("cutler.toml");
        candidates.push(candidate2);
    }

    candidates.push(PathBuf::from("config.toml"));

    // return the first candidate that exists
    // might lead to a prompt to create an example config
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.to_owned();
        }
    }
    candidates
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

/// Helper: Read and parse the configuration file at a given path.
pub fn load_config(path: &Path) -> Result<Value, anyhow::Error> {
    let content = fs::read_to_string(path)?;
    let parsed: Value = content.parse::<Value>()?;
    Ok(parsed)
}
