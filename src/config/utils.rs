use std::path::PathBuf;

use anyhow::anyhow;
use tokio::fs;

use crate::util::{
    globals::should_dry_run,
    logging::{LogLevel, print_log},
};

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
