use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;

use crate::{
    commands::Runnable,
    config::{
        loader::{get_config_path, load_config},
        remote::{
            REMOTE_CONFIG, RemoteConfig, fetch_remote_config, save_merge_local_remote_config,
        },
    },
    util::{
        globals::should_dry_run,
        io::confirm_action,
        logging::{BOLD, GREEN, LogLevel, RESET, print_log},
    },
};

#[derive(Debug, Default, Args)]
pub struct FetchCmd;

#[async_trait]
impl Runnable for FetchCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path().await;
        let dry_run = should_dry_run();

        let local_doc = load_config(false).await?;

        // parse [remote] section
        let remote = match RemoteConfig::from_toml(&local_doc) {
            Some(cfg) => cfg,
            None => bail!("No [remote] section found in config. Add one to use remote sync."),
        };

        // fetch remote config
        fetch_remote_config(remote.url).await?;
        let remote_doc = REMOTE_CONFIG
            .get()
            .cloned()
            .expect("Could not load remote configuration.");

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
            print_log(LogLevel::Fruitful, "No differences made, no files hurt.");
            return Ok(());
        } else {
            print_log(
                LogLevel::Warning,
                "Differences between local and remote config:",
            );
            for line in &changes {
                print_log(LogLevel::Warning, &format!("  {line}"));
            }
        }

        // prompt user to proceed (unless dry-run)
        if !dry_run && !confirm_action("Apply remote config (overwrite local config)?")? {
            print_log(LogLevel::Warning, "Sync aborted by user.");
            return Ok(());
        }

        // overwrite local config with remote config (or just print in dry-run)
        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would overwrite {cfg_path:?} with remote config."),
            );
        } else {
            save_merge_local_remote_config().await?;

            print_log(
                LogLevel::Fruitful,
                &format!("{GREEN}Local config updated from remote!{RESET}"),
            );
        }

        Ok(())
    }
}
