use std::{env, path::PathBuf, sync::OnceLock};

use anyhow::{Context, Result, bail};
use tokio::fs;
use toml::Value;
use toml_edit::{DocumentMut, Item};

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
        if fs::try_exists(candidate).await.unwrap() {
            return candidate.to_owned();
        }
    }

    // if none exist, always return $HOME/.config/cutler/config.toml if HOME is set
    // else fallback to ~/.config/cutler/config.toml
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

/// Variable to cache the configuration file content for the process lifetime.
static CONFIG_CONTENT: OnceLock<String> = OnceLock::new();

/// Helper for: load_config(), load_config_mut()
/// Read and cache the configuration file content for the process lifetime.
async fn get_config_content() -> Result<(String, PathBuf), anyhow::Error> {
    get_config_content_inner(true).await
}

/// Helper for: load_config_detached()
/// Read the configuration file content without caching.
async fn get_config_content_detached() -> Result<(String, PathBuf), anyhow::Error> {
    get_config_content_inner(false).await
}

/// Internal helper to read config content, optionally using cache.
async fn get_config_content_inner(use_cache: bool) -> Result<(String, PathBuf), anyhow::Error> {
    let path = get_config_path().await;
    if !fs::try_exists(&path).await.unwrap() {
        bail!("No config file found at {path:?}.\nPlease start by creating one with `cutler init`.")
    }

    if use_cache {
        // try to get from cache
        if let Some(content) = CONFIG_CONTENT.get() {
            return Ok((content.clone(), path));
        }
    }

    let content = fs::read_to_string(&path)
        .await
        .with_context(|| format!("Failed to read config file at {path:?}"))?;

    if use_cache {
        // cache it
        let _ = CONFIG_CONTENT.set(content.clone());
    }

    Ok((content, path))
}

/// Read and parse the configuration file at a given path, using cached content.
pub async fn load_config(lock_check: bool) -> Result<Value, anyhow::Error> {
    load_config_common(get_config_content().await?, lock_check)
}

/// Read and parse the configuration file at a given path, without using cached content.
pub async fn load_config_detached(lock_check: bool) -> Result<Value, anyhow::Error> {
    load_config_common(get_config_content_detached().await?, lock_check)
}

/// Mutably read and parse the configuration file at a given path.
pub async fn load_config_mut(lock_check: bool) -> Result<DocumentMut, anyhow::Error> {
    load_config_mut_common(get_config_content().await?, lock_check)
}

/// Common logic for loading config as Value.
fn load_config_common(
    (content, path): (String, PathBuf),
    lock_check: bool,
) -> Result<Value, anyhow::Error> {
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

/// Common logic for loading config as DocumentMut.
fn load_config_mut_common(
    (content, path): (String, PathBuf),
    lock_check: bool,
) -> Result<DocumentMut, anyhow::Error> {
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
