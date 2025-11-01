// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use tokio::process::Command;

use crate::{
    brew::{
        core::{compare_brew_state, ensure_brew},
        types::BrewDiff,
    },
    cli::atomic::{should_be_quiet, should_dry_run},
    commands::Runnable,
    config::core::Config,
    log_cute, log_dry, log_err, log_info, log_warn,
};

#[derive(Debug, Args)]
pub struct BrewInstallCmd;

#[async_trait]
impl Runnable for BrewInstallCmd {
    async fn run(&self) -> Result<()> {
        let dry_run = should_dry_run();

        let config = Config::load(true).await?;
        let brew_cfg = config
            .brew
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No [brew] section found in config"))?;

        // ensure homebrew installation
        ensure_brew().await?;

        // check the current brew state, including taps, formulae, and casks
        let brew_diff = match compare_brew_state(brew_cfg).await {
            Ok(diff) => {
                if !diff.extra_formulae.is_empty() {
                    log_warn!(
                        "Extra installed formulae not in config: {:?}",
                        diff.extra_formulae
                    );
                }
                if !diff.extra_casks.is_empty() {
                    log_warn!(
                        "Extra installed casks not in config: {:?}",
                        diff.extra_casks
                    );
                }
                if !diff.extra_taps.is_empty() {
                    log_warn!("Extra taps not in config: {:?}", diff.extra_taps,);
                }
                if !diff.extra_formulae.is_empty() || !diff.extra_casks.is_empty() {
                    log_warn!(
                        "Run `cutler brew backup` to synchronize your config with the system.\n",
                    );
                }
                diff
            }
            Err(e) => {
                log_err!("Could not check Homebrew status: {e}",);
                // If we cannot compare the state, treat as if nothing is missing.
                BrewDiff {
                    missing_formulae: vec![],
                    extra_formulae: vec![],
                    missing_casks: vec![],
                    extra_casks: vec![],
                    missing_taps: vec![],
                    extra_taps: vec![],
                }
            }
        };

        // tap only the missing taps reported by BrewDiff
        if !brew_diff.missing_taps.is_empty() {
            for tap in brew_diff.missing_taps.iter() {
                if dry_run {
                    log_dry!("Would tap {tap}");
                } else {
                    log_info!("Tapping: {tap}");
                    let status = Command::new("brew").arg("tap").arg(tap).status().await?;

                    if !status.success() {
                        log_err!("Failed to tap: {tap}");
                    }
                }
            }
        }

        if !brew_diff.missing_formulae.is_empty() || !brew_diff.missing_casks.is_empty() {
            log_info!("Pre-downloading all formulae and casks...");
        } else {
            log_cute!("No formulae or casks to download/install.");
            return Ok(());
        }

        // handle all of dry-run in this single block
        if dry_run {
            brew_diff.missing_formulae.iter().for_each(|formula| {
                log_info!("Would fetch formula: {formula}");
            });
            brew_diff.missing_casks.iter().for_each(|cask| {
                log_dry!("Would fetch cask: {cask}");
            });
            return Ok(());
        }

        let fetched = fetch_all(&brew_diff.missing_formulae, &brew_diff.missing_casks).await;

        // sequentially install only the successfully fetched items
        install_all(fetched.formulae, false).await?;
        install_all(fetched.casks, true).await?;

        Ok(())
    }
}

/// Represents the result of fetching formulae and casks.
pub struct FetchedThings {
    pub formulae: Vec<String>,
    pub casks: Vec<String>,
}

/// Downloads all formulae/casks before installation, sequentially.
/// Returns only the successfully fetched formulae and casks.
async fn fetch_all(formulae: &[String], casks: &[String]) -> FetchedThings {
    let quiet = should_be_quiet();

    // create new vectors
    let mut fetched_formulae = Vec::new();
    let mut fetched_casks = Vec::new();
    let mut failed_formulae = Vec::new();
    let mut failed_casks = Vec::new();

    // fetch formulae sequentially
    for name in formulae {
        let mut cmd = Command::new("brew");
        cmd.arg("fetch").arg(name);

        if !quiet {
            log_info!("Fetching formula: {name}");
        } else {
            cmd.arg("--quiet");
        }

        match cmd.status().await {
            Ok(status) if status.success() => fetched_formulae.push(name.clone()),
            _ => failed_formulae.push(name.clone()),
        }
    }

    // fetch casks sequentially
    for name in casks {
        let mut cmd = Command::new("brew");
        cmd.arg("fetch").arg("--cask").arg(name);

        if !quiet {
            log_info!("Fetching cask: {name}");
        } else {
            cmd.arg("--quiet");
        }

        match cmd.status().await {
            Ok(status) if status.success() => fetched_casks.push(name.clone()),
            _ => failed_casks.push(name.clone()),
        }
    }

    // warn user about failed formulae and casks
    if !failed_formulae.is_empty() {
        log_warn!("Failed to fetch formulae: {failed_formulae:?}",);
    }
    if !failed_casks.is_empty() {
        log_warn!("Failed to fetch casks: {failed_casks:?}",);
    }
    if !failed_formulae.is_empty() || !failed_casks.is_empty() {
        log_warn!("Some software failed to download and won't be installed.",);
    }

    FetchedThings {
        formulae: fetched_formulae,
        casks: fetched_casks,
    }
}

/// Install formulae/casks sequentially.
/// The argument is a vector of argslices, representing the arguments to the `brew install` subcommand.
async fn install_all(install_tasks: Vec<String>, cask: bool) -> anyhow::Result<()> {
    for task in install_tasks {
        log_info!("Installing: {task}");

        let status = Command::new("brew")
            .arg("install")
            .arg(if cask { "--cask" } else { "--formula" })
            .arg(&task)
            .status()
            .await?;

        if !status.success() {
            log_err!("Failed to install: {task}");
        }
    }
    Ok(())
}
