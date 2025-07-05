use crate::config::loader::{create_config, get_config_path};
use crate::util::io::confirm_action;
use crate::util::logging::{LogLevel, print_log};
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

/// Ensures the config file exists, or prompts to create it (runs init if needed).
/// Returns Ok(Some(path)) if config exists (or was created), Ok(None) if user aborted.
pub async fn ensure_config_exists_or_init() -> Result<Option<PathBuf>> {
    let config_path = get_config_path().await;
    if fs::try_exists(&config_path).await.unwrap() {
        return Ok(Some(config_path));
    }
    print_log(
        LogLevel::Warning,
        &format!("Config not found at {config_path:?}"),
    );
    if confirm_action("Create a new config?")? {
        create_config(&config_path).await?;
        Ok(Some(config_path))
    } else {
        print_log(LogLevel::Warning, "No config; aborting.");
        Ok(None)
    }
}
