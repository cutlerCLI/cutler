// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::Command;
use crate::cli::args::BrewSubcmd;
use crate::config::core::Config;
use crate::config::remote::RemoteConfigManager;
use crate::{log_err, log_info, log_warn};

/// Perform remote config auto-sync if enabled in [remote] and internet is available.
/// This should be called early in main().
pub async fn try_auto_sync(command: &crate::cli::Command, local_config: &mut Config) {
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
        | Command::Config(_) => {
            return;
        }
        _ => {}
    }

    if local_config.load(true).await.is_err() {
        return;
    };

    // start
    let remote = local_config.remote.clone().unwrap_or_default();
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
