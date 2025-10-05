// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{loader::Config, path::get_config_path, remote::RemoteConfigManager},
    util::{
        io::confirm,
        logging::{BOLD, LogLevel, RESET, print_log},
    },
};

#[derive(Debug, Args)]
pub struct FetchCmd {
    /// Fetches the configuration regardless of whether the configuration is equal value-wise..
    #[arg(short, long)]
    force: bool,
}

#[async_trait]
impl Runnable for FetchCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path().await;
        let dry_run = should_dry_run();

        let local_config = Config::load().await?;

        // parse [remote] section
        let remote_mgr = match RemoteConfigManager::from_config(&local_config) {
            Some(cfg) => cfg,
            None => bail!("No [remote] section found in config. Add one to use remote sync."),
        };

        // fetch remote config
        remote_mgr.fetch().await?;

        if !self.force {
            let remote_config = remote_mgr.get_parsed()?;

            // comparison begins
            let mut changes = Vec::new();

            // Compare fields between local_config and remote_config
            // Example: compare brew, remote, vars, etc.
            if local_config.brew.as_ref() != remote_config.brew.as_ref() {
                changes.push(format!("{BOLD}brew{RESET}: (changed)"));
            }
            if local_config.remote.as_ref() != remote_config.remote.as_ref() {
                changes.push(format!("{BOLD}remote{RESET}: (changed)"));
            }
            if local_config.vars.as_ref() != remote_config.vars.as_ref() {
                changes.push(format!("{BOLD}vars{RESET}: (changed)"));
            }
            // Add more comparisons as needed for your config structure

            if changes.is_empty() {
                print_log(
                    LogLevel::Fruitful,
                    "No changes found so skipping. Use -f to fetch forcefully.",
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

            if changes.is_empty() {
                print_log(
                    LogLevel::Fruitful,
                    "No changes found so skipping. Use -f to fetch forcefully.",
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
        }

        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would overwrite {cfg_path:?} with remote config."),
            );
        } else {
            remote_mgr.save().await?;

            print_log(LogLevel::Fruitful, "Local config updated from remote!");
        }

        Ok(())
    }
}
