// SPDX-License-Identifier: Apache-2.0

use crate::cli::Command;
use crate::cli::args::BrewSubcmd;
use crate::config::remote::RemoteConfigManager;
use crate::{
    config::{loader::Config, path::get_config_path},
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

    let cfg_path = get_config_path().await;
    if !cfg_path.exists() {
        return;
    }

    let local_config = match Config::load().await {
        Ok(cfg) => cfg,
        Err(e) => {
            print_log(
                LogLevel::Warning,
                &format!("Failed to load config for auto-sync: {e}"),
            );
            return;
        }
    };

    // start
    let remote_mgr = RemoteConfigManager::from_config(&local_config);

    if let Some(remote_mgr) = remote_mgr {
        if remote_mgr.remote.autosync.unwrap_or_default() {
            match remote_mgr.fetch().await {
                Ok(()) => {
                    if let Err(e) = remote_mgr.save(&cfg_path).await {
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
}
