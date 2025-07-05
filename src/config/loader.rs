use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use tokio::fs;
use toml::Value;
use toml_edit::{DocumentMut, Item};

use crate::util::{
    globals::should_dry_run,
    logging::{LogLevel, print_log},
};

/// Returns the path to the configuration file by checking several candidate locations.
pub async fn get_config_path() -> PathBuf {
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

    candidates.push(PathBuf::from("cutler.toml"));

    // return the first candidate that exists
    // might lead to a prompt to create an example config
    for candidate in &candidates {
        if fs::try_exists(candidate).await.unwrap() {
            return candidate.to_owned();
        }
    }
    candidates
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("cutler.toml"))
}

/// Read and parse the configuration file at a given path.
pub async fn load_config(path: &Path, lock_check: bool) -> Result<Value, anyhow::Error> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read config file at {path:?}"))?;
    let parsed: Value = content.parse::<Value>().with_context(|| {
        format!(
            "Failed to parse TOML at {path:?}. Please check for syntax errors or invalid structure."
        )
    })?;

    // handle optional locking
    if parsed.get("lock").and_then(Value::as_bool).unwrap_or(false) && lock_check {
        bail!("The config file is locked. Run `cutler config unlock` to unlock.");
    }
    Ok(parsed)
}

/// Same as load_config, but returns a mutable DocumentMut instance.
pub async fn load_config_mut(path: &Path, lock_check: bool) -> Result<DocumentMut, anyhow::Error> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read config file at {path:?}"))?;
    let parsed: DocumentMut = content.parse::<DocumentMut>().with_context(|| {
        format!(
            "Failed to parse TOML at {path:?}. Please check for syntax errors or invalid structure."
        )
    })?;

    // handle optional locking
    if parsed.get("lock").and_then(Item::as_bool).unwrap_or(false) && lock_check {
        bail!("The config file is locked. Run `cutler config unlock` to unlock.");
    }

    Ok(parsed)
}

/// Creates a new configuration file (uses complete.toml template).
pub async fn create_config(config_path: &PathBuf) -> Result<(), anyhow::Error> {
    let dry_run = should_dry_run();

    // ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would create directory: {parent:?}"),
            );
        } else {
            print_log(LogLevel::Info, &format!("Creating parent dir: {parent:?}"));
            fs::create_dir_all(parent).await?;
        }
    }

    // TOML template
    let default_cfg = include_str!("../../examples/complete.toml");

    if dry_run {
        print_log(
            LogLevel::Dry,
            &format!("Would write configuration to {config_path:?}"),
        );
        print_log(
            LogLevel::Dry,
            &format!("Configuration content:\n{default_cfg}"),
        );

        Ok(())
    } else {
        fs::write(&config_path, default_cfg)
            .await
            .map_err(|e| anyhow!("Failed to write configuration to {:?}: {}", config_path, e))?;

        print_log(
            LogLevel::Fruitful,
            &format!("Config created at {config_path:?}, Review and customize it before applying."),
        );

        Ok(())
    }
}
