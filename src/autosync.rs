use crate::cli::Command;
use crate::commands::{CheckUpdateCmd, FetchCmd, SelfUpdateCmd};
use crate::config::loader::{get_config_path, load_config_detached};
use crate::config::remote::{RemoteConfig, fetch_remote_config, save_remote_config};
use crate::util::logging::{LogLevel, print_log};

/// Perform remote config auto-sync if enabled in [remote] and internet is available.
/// This should be called early in main().
pub async fn try_auto_sync(command: &crate::cli::Command) {
    match command {
        Command::Fetch(FetchCmd)
        | Command::SelfUpdate(SelfUpdateCmd)
        | Command::CheckUpdate(CheckUpdateCmd) => {
            return;
        }
        _ => {}
    }

    let cfg_path = get_config_path().await;
    if !cfg_path.exists() {
        return;
    }

    // use raw-reading, bypassing loader.rs
    // this is to avoid caching a possible 'old' config scenario
    let local_doc = match load_config_detached(true).await {
        Ok(doc) => doc,
        Err(e) => {
            print_log(
                LogLevel::Warning,
                &format!("Failed to load config for auto-sync: {e}"),
            );
            return;
        }
    };

    // start
    let remote_cfg = RemoteConfig::from_toml(&local_doc);
    if let Some(remote_cfg) = remote_cfg {
        if remote_cfg.autosync {
            match fetch_remote_config(remote_cfg.url).await {
                Ok(()) => {
                    if let Err(e) = save_remote_config().await {
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
                LogLevel::Warning,
                "Remote config auto-sync disabled. To manually sync, run `cutler fetch`.",
            );
        }
    }
}
