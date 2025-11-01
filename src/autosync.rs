// SPDX-License-Identifier: GPL-3.0-or-later

use crate::cli::Command;
use crate::cli::args::BrewSubcmd;
use crate::config::remote::RemoteConfigManager;
use crate::{
    config::core::Config,
    util::logging::{LogLevel, print_log},
};

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

    if !Config::is_loadable().await {
        return;
    }

    let local_config = match Config::load(true).await {
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
                    print_log(
                        LogLevel::Warning,
                        &format!("Failed to save remote config after auto-sync: {e}"),
                    );
                }
            }
            Err(e) => {
                print_log(
                    LogLevel::Warning,
                    &format!("Remote config auto-sync failed: {e}"),
                );
            }
        }
    } else {
        print_log(
            LogLevel::Info,
            "Skipping auto-sync since disabled in config.",
        );
    }
}
