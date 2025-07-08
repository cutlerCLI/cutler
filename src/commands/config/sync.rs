use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    commands::Runnable,
    config::{
        load_config,
        loader::get_config_path,
        remote::{RemoteConfig, fetch_remote_config},
    },
    util::{
        globals::should_dry_run,
        io::confirm_action,
        logging::{BOLD, GREEN, LogLevel, RESET, print_log},
    },
};

/// Sync your local config with the remote config defined in [remote].
#[derive(Debug, Default, Args)]
pub struct ConfigSyncCmd;

#[async_trait]
impl Runnable for ConfigSyncCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path().await;
        let dry_run = should_dry_run();

        let local_doc = load_config(false).await?;

        // parse [remote] section
        let remote_cfg = match RemoteConfig::from_toml(&local_doc) {
            Some(cfg) => cfg,
            None => bail!("No [remote] section found in config. Add one to use remote sync."),
        };

        // fetch remote config
        let remote_doc = match fetch_remote_config(remote_cfg.url).await {
            Ok(val) => val,
            Err(e) => bail!("Failed to fetch remote config: {e}"),
        };

        // comparison begins
        let mut changes = Vec::new();

        let local_table = match local_doc.as_table() {
            Some(tbl) => tbl,
            None => bail!("Local config is not a TOML table."),
        };
        let remote_table = match remote_doc.as_table() {
            Some(tbl) => tbl,
            None => bail!("Remote config is not a TOML table."),
        };

        for (k, v) in remote_table.iter() {
            if !local_table.contains_key(k) {
                changes.push(format!("{BOLD}{k}{RESET}: (new)"));
            } else if local_table[k].to_string() != v.to_string() {
                changes.push(format!("{BOLD}{k}{RESET}: (changed)"));
            }
        }

        for k in local_table.keys() {
            if !remote_table.contains_key(k) {
                changes.push(format!("{BOLD}{k}{RESET}: (removed in remote)"));
            }
        }

        if changes.is_empty() {
            print_log(
                LogLevel::Info,
                "No differences found between local and remote config.",
            );
        } else {
            print_log(
                LogLevel::Info,
                "Differences between local and remote config:",
            );
            for line in &changes {
                println!("  {line}");
            }
        }

        // prompt user to proceed (unless dry-run)
        if !dry_run && !confirm_action("Apply remote config (overwrite local config)?")? {
            print_log(LogLevel::Warning, "Sync aborted by user.");
            return Ok(());
        }

        // overwrite local config with remote config (or just print in dry-run)
        let remote_text = remote_doc.to_string();
        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would overwrite {cfg_path:?} with remote config:"),
            );
            println!("{remote_text}");
        } else {
            fs::write(&cfg_path, &remote_text).await?;
            print_log(
                LogLevel::Fruitful,
                &format!("{GREEN}Local config updated from remote!{RESET}"),
            );
        }

        Ok(())
    }
}
