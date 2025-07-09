use crate::commands::FetchCmd;
use crate::config::loader::get_config_path;
use crate::config::remote::{RemoteConfig, merge_remote_config};
use crate::util::logging::{LogLevel, print_log};
use tokio::fs;
use toml::Value;

/// Perform remote config auto-sync if enabled in [remote] and internet is available.
/// This should be called early in main().
pub async fn try_auto_sync(command: &crate::cli::Command) {
    if matches!(command, crate::cli::Command::Fetch(FetchCmd)) {
        return;
    }

    let cfg_path = get_config_path().await;
    if !cfg_path.exists() {
        return;
    }

    // use raw-reading, bypassing loader.rs
    // this is to avoid caching a possible 'old' config scenario
    let local_doc = match fs::read_to_string(cfg_path).await {
        Ok(doc) => doc,
        Err(_) => {
            print_log(
                LogLevel::Warning,
                &format!("Could not read config for invoking try_auto_sync!"),
            );
            return;
        }
    }
    .parse::<Value>()
    .unwrap();

    // start
    let remote_cfg = RemoteConfig::from_toml(&local_doc);
    if let Some(remote_cfg) = remote_cfg {
        if remote_cfg.autosync {
            match remote_cfg.fetch().await {
                Ok(remote_val) => {
                    // preserve/merge [remote]
                    let remote_text = merge_remote_config(&local_doc, &remote_val)
                        .as_table()
                        .unwrap()
                        .to_string();
                    let cfg_path = get_config_path().await;

                    // finally write to disk
                    if let Err(e) = fs::write(&cfg_path, remote_text).await {
                        print_log(
                            LogLevel::Warning,
                            &format!("Failed to auto-sync remote config: {e}"),
                        );
                    } else {
                        print_log(LogLevel::Info, "Auto-synced config from remote.");
                    }
                }
                Err(e) => {
                    print_log(
                        LogLevel::Warning,
                        &format!("Remote config auto-sync failed: {e}"),
                    );
                }
            }
        }
    }
}
