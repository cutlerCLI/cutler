// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::Command;
use crate::cli::args::BrewSubcmd;
use crate::config::core::Config;
use crate::config::path::get_config_path;
use crate::config::remote::RemoteConfigManager;
use crate::{log_err, log_info, log_warn};

/// Perform remote config auto-sync if enabled in [remote] and internet is available.
/// This should be called early in main().
pub async fn try_auto_sync(command: &crate::cli::Command) {
    match command {
        Command::Fetch(_)
        | Command::Brew {
            command: BrewSubcmd::Backup(_),
        }
        | Command::SelfUpdate(_)
        | Command::CheckUpdate(_)
        | Command::Cookbook(_)
        | Command::Completion(_)
        | Command::Reset(_)
        | Command::Init(_)
        | Command::Config { .. } => {
            return;
        }
        _ => {}
    }

    let config_path = get_config_path().await.unwrap_or_default();
    if !config_path.try_exists().unwrap_or(false) {
        return;
    }

    let local_config = match Config::new(config_path).load(true).await {
        Ok(cfg) => cfg,
        Err(_) => {
            // Loading error handling is managed by later loads.
            // Skipped for this one, otherwise the error would double.
            return;
        }
    };

    // start
    let remote = local_config.remote.unwrap_or_default();
    let remote_mgr = RemoteConfigManager::new(remote.url);

    if remote.autosync.unwrap_or_default() {
        match remote_mgr.fetch().await {
            Ok(()) => {
                if let Err(e) = remote_mgr.save().await {
                    log_err!("Failed to save remote config after auto-sync: {e}");
                }
            }
            Err(e) => {
                log_warn!("Remote config auto-sync failed: {e}",);
            }
        }
    } else {
        log_info!("Skipping auto-sync since disabled in config.",);
    }
}
