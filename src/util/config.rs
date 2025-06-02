use crate::config::loader::get_config_path;
use crate::util::io::confirm_action;
use crate::util::logging::{LogLevel, print_log};
use anyhow::Result;
use std::path::PathBuf;

/// Ensures the config file exists, or prompts to create it (runs init if needed).
/// Returns Ok(Some(path)) if config exists (or was created), Ok(None) if user aborted.
pub async fn ensure_config_exists_or_init(
    verbose: bool,
    dry_run: bool,
    force_basic: bool,
) -> Result<Option<PathBuf>> {
    let config_path = get_config_path();
    if config_path.exists() {
        return Ok(Some(config_path));
    }
    print_log(
        LogLevel::Info,
        &format!("Config not found at {:?}", config_path),
    );
    if confirm_action("Create a new basic config?")? {
        crate::commands::init::run(force_basic, verbose, dry_run, false).await?;
        Ok(Some(config_path))
    } else {
        print_log(LogLevel::Warning, "No config; aborting.");
        Ok(None)
    }
}
