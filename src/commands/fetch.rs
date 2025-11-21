// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;

use crate::{
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{core::Config, remote::RemoteConfigManager},
    log_cute, log_dry, log_warn,
    util::{
        io::confirm,
        logging::{BOLD, RESET},
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
    async fn run(&self, local_config: &mut Config) -> Result<()> {
        let dry_run = should_dry_run();

        // prepare local config for comparison
        local_config.load(true).await?;

        // parse [remote] section
        let remote_mgr = if let Some(ref remote) = local_config.remote {
            RemoteConfigManager::new(remote.clone().url)
        } else {
            bail!("No URL found in [remote] of config. Add one to use remote sync.")
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
                log_cute!("No changes found so skipping. Use -f to fetch forcefully.",);
                return Ok(());
            }
            log_warn!("Differences between local and remote config:",);
            for line in &changes {
                log_warn!("  {line}");
            }

            if changes.is_empty() {
                log_cute!("No changes found so skipping. Use -f to fetch forcefully.",);
                return Ok(());
            }
            log_warn!("Differences between local and remote config:",);
            for line in &changes {
                log_warn!("  {line}");
            }

            // prompt user to proceed (unless dry-run)
            if !dry_run && !confirm("Apply remote config (overwrite local config)?") {
                log_warn!("Sync aborted by user.");
                return Ok(());
            }
        }

        if dry_run {
            log_dry!(
                "Would overwrite {:?} with remote config.",
                local_config.path
            );
        } else {
            remote_mgr.save().await?;

            log_cute!("Local config updated from remote!");
        }

        Ok(())
    }
}
