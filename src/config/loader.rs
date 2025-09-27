// SPDX-License-Identifier: Apache-2.0

use std::{path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result, bail};
use tokio::fs;
use toml::{Table, Value};
use toml_edit::{DocumentMut, Item};

use crate::config::path::get_config_path;

/// Variable to cache the configuration file content for the process lifetime.
static CONFIG_CONTENT: OnceLock<String> = OnceLock::new();

/// Helper for: load_config(), load_config_mut()
/// Read and cache the configuration file content for the process lifetime.
async fn get_config_content() -> Result<(String, PathBuf)> {
    let path = get_config_path().await;

    if path.try_exists()? {
        bail!("No config file found at {path:?}.\nPlease start by creating one with `cutler init`.")
    }

    // try to get from cache
    if let Some(content) = CONFIG_CONTENT.get() {
        return Ok((content.clone(), path));
    }

    let content = fs::read_to_string(&path)
        .await
        .with_context(|| format!("Failed to read config file at {path:?}"))?;

    // cache it
    let _ = CONFIG_CONTENT.set(content.clone());

    Ok((content, path))
}

/// Read and parse the configuration file at a given path.
pub async fn load_config(lock_check: bool) -> Result<Table> {
    let (content, path) = get_config_content().await?;

    let parsed: Table = content.parse::<Table>().with_context(|| {
        format!("Failed to parse TOML at {path:?}. Please check for syntax errors.")
    })?;

    // handle optional locking
    if parsed.get("lock").and_then(Value::as_bool).unwrap_or(false) && lock_check {
        bail!("The config file is locked. Run `cutler config unlock` to unlock.");
    }

    Ok(parsed)
}

/// Mutably read and parse the configuration file at a given path.
pub async fn load_config_mut(lock_check: bool) -> Result<DocumentMut> {
    let (content, path) = get_config_content().await?;

    let parsed: DocumentMut = content.parse::<DocumentMut>().with_context(|| {
        format!("Failed to parse TOML at {path:?}. Please check for syntax errors.")
    })?;

    // handle optional locking
    if parsed.get("lock").and_then(Item::as_bool).unwrap_or(false) && lock_check {
        bail!("The config file is locked. Run `cutler config unlock` to unlock.");
    }

    Ok(parsed)
}

/// Detached version of load_config: does not cache the result and does not interact with the OnceLock.
pub async fn load_config_detached(lock_check: bool) -> Result<Table> {
    let path = get_config_path().await;

    if path.try_exists()? {
        bail!("No config file found at {path:?}.\nPlease start by creating one with `cutler init`.")
    }

    let content = fs::read_to_string(&path)
        .await
        .with_context(|| format!("Failed to read config file at {path:?}"))?;

    let parsed: Table = content.parse::<Table>().with_context(|| {
        format!("Failed to parse TOML at {path:?}. Please check for syntax errors.")
    })?;

    // handle optional locking
    if parsed.get("lock").and_then(Value::as_bool).unwrap_or(false) && lock_check {
        bail!("The config file is locked. Run `cutler config unlock` to unlock.");
    }

    Ok(parsed)
}
