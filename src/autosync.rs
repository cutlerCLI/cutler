use crate::cli::args::ConfigSubcmd;
use crate::config::remote::{RemoteConfig, fetch_remote_config};
use crate::config::{get_config_path, load_config};
use crate::util::logging::{LogLevel, print_log};
use tokio::fs;

/// Checks if auto-sync should run (not during config sync command).
pub fn should_auto_sync(command: &crate::cli::Command) -> bool {
    match command {
        crate::cli::Command::Config { command } => {
            matches!(command, ConfigSubcmd::Sync(_))
        }
        _ => false,
    }
}

/// Perform remote config auto-sync if enabled in [remote] and internet is available.
/// This should be called early in main().
pub async fn try_auto_sync(command: &crate::cli::Command) {
    if should_auto_sync(command) {
        return;
    }

    let local_doc = match load_config(false).await {
        Ok(doc) => doc,
        Err(_) => return,
    };

    let remote_cfg = RemoteConfig::from_toml(&local_doc);
    if let Some(remote_cfg) = remote_cfg {
        if remote_cfg.update_on_cmd {
            match fetch_remote_config(remote_cfg.url).await {
                Ok(remote_val) => {
                    let remote_text = remote_val.to_string();
                    let cfg_path = get_config_path().await;

                    if let Err(e) = fs::write(&cfg_path, &remote_text).await {
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
