// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use toml::Table;

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{loader::load_config, path::get_config_path, remote::RemoteConfigManager},
    util::{
        io::confirm,
        logging::{BOLD, LogLevel, RESET, print_log},
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
        let remote_mgr = match RemoteConfigManager::from_toml(&local_doc) {
            Some(cfg) => cfg,
            None => bail!("No [remote] section found in config. Add one to use remote sync."),
        };

        // fetch remote config
        remote_mgr.fetch().await?;
        let remote_doc = remote_mgr.get().cloned()?.parse::<Table>()?;

        // comparison begins
        let mut changes = Vec::new();

        for (k, v) in remote_doc.iter() {
            if !local_doc.contains_key(k) {
                changes.push(format!("{BOLD}{k}{RESET}: (new)"));
            } else if local_doc[k].to_string() != v.to_string() {
                changes.push(format!("{BOLD}{k}{RESET}: (changed)"));
            }
        }

        for k in local_doc.keys() {
            if !remote_doc.contains_key(k) {
                changes.push(format!("{BOLD}{k}{RESET}: (removed in remote)"));
            }
        }

        if changes.is_empty() {
            print_log(
                LogLevel::Fruitful,
                "No changes found between remote & local configs.",
            );
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
        if !dry_run && !confirm("Apply remote config (overwrite local config)?") {
            print_log(LogLevel::Warning, "Sync aborted by user.");
            return Ok(());
        }

        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would overwrite {cfg_path:?} with remote config."),
            );
        } else {
            remote_mgr.save(&cfg_path).await?;

            print_log(LogLevel::Fruitful, "Local config updated from remote!");
        }

        Ok(())
    }
}
