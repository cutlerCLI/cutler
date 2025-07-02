use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use tokio::process::Command;

use crate::{
    brew::{
        types::BrewDiff,
        utils::{compare_brew_state, ensure_brew},
    },
    commands::Runnable,
    config::{get_config_path, load_config},
    util::{
        globals::{is_verbose, should_dry_run},
        logging::{LogLevel, print_log},
    },
};

#[derive(Debug, Default, Args)]
pub struct BrewInstallCmd;

#[async_trait]
impl Runnable for BrewInstallCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path();
        let dry_run = should_dry_run();

        if !cfg_path.exists() {
            print_log(
                LogLevel::Error,
                "No config file found. Run `cutler init` to start.",
            );
            return Ok(());
        }

        // ensure homebrew installation
        ensure_brew().await?;

        let config = load_config(&cfg_path, true).await?;
        let brew_cfg = config
            .get("brew")
            .and_then(|i| i.as_table())
            .context("No [brew] table found in config")?;

        // check the current brew state, including taps, formulae, and casks
        let brew_diff = match compare_brew_state(brew_cfg).await {
            Ok(diff) => {
                if !diff.extra_formulae.is_empty() {
                    print_log(
                        LogLevel::Warning,
                        &format!(
                            "Extra installed formulae not in config: {:?}",
                            diff.extra_formulae
                        ),
                    );
                }
                if !diff.extra_casks.is_empty() {
                    print_log(
                        LogLevel::Warning,
                        &format!(
                            "Extra installed casks not in config: {:?}",
                            diff.extra_casks
                        ),
                    );
                }
                if !diff.extra_taps.is_empty() {
                    print_log(
                        LogLevel::Warning,
                        &format!("Extra taps not in config: {:?}", diff.extra_taps),
                    );
                }
                if !diff.extra_formulae.is_empty() || !diff.extra_casks.is_empty() {
                    print_log(
                        LogLevel::Warning,
                        "Run `cutler brew backup` to synchronize your config with the system.\n",
                    );
                }
                diff
            }
            Err(e) => {
                print_log(
                    LogLevel::Error,
                    &format!("Could not check Homebrew status: {e}"),
                );
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
                    print_log(LogLevel::Dry, &format!("Would tap {tap}"));
                } else {
                    print_log(LogLevel::Info, &format!("Tapping: {tap}"));
                    let status = Command::new("brew")
                        .arg("tap")
                        .arg(tap)
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .stdin(std::process::Stdio::inherit())
                        .status()
                        .await?;
                    if !status.success() {
                        print_log(LogLevel::Error, &format!("Failed to tap: {tap}"));
                    }
                }
            }
        }

        if !brew_diff.missing_formulae.is_empty() || !brew_diff.missing_casks.is_empty() {
            print_log(LogLevel::Info, "Pre-downloading all formulae and casks...");
        } else {
            print_log(LogLevel::Info, "No formulae or casks to download/install.");
            return Ok(());
        }

        // handle all of dry-run in this single block
        if dry_run {
            brew_diff.missing_formulae.iter().for_each(|formula| {
                print_log(LogLevel::Dry, &format!("Would fetch formula: {formula}"));
            });
            brew_diff.missing_casks.iter().for_each(|cask| {
                print_log(LogLevel::Dry, &format!("Would fetch cask: {cask}"));
            });
            return Ok(());
        }

        let fetched = fetch_all(&brew_diff.missing_formulae, &brew_diff.missing_casks).await;

        // build install tasks only for successfully fetched items
        let mut install_args: Vec<Vec<String>> = Vec::new();

        for name in fetched.formulae {
            install_args.push(vec!["--formula".to_string(), name]);
        }
        for name in fetched.casks {
            install_args.push(vec!["--cask".to_string(), name]);
        }

        // sequentially install only the successfully fetched items
        install_all(install_args).await?;

        Ok(())
    }
}

/// Represents the result of fetching formulae and casks.
pub struct FetchedThings {
    pub formulae: Vec<String>,
    pub casks: Vec<String>,
    pub failed_formulae: Vec<String>,
    pub failed_casks: Vec<String>,
}

/// Downloads all formulae/casks before installation, sequentially.
/// Returns only the successfully fetched formulae and casks.
async fn fetch_all(formulae: &[String], casks: &[String]) -> FetchedThings {
    let verbose = is_verbose();

    // create new vectors
    let mut fetched_formulae = Vec::new();
    let mut fetched_casks = Vec::new();
    let mut failed_formulae = Vec::new();
    let mut failed_casks = Vec::new();

    // Fetch formulae sequentially
    for name in formulae {
        let mut cmd = Command::new("brew");
        cmd.arg("fetch").arg(name);

        if verbose {
            print_log(LogLevel::Info, &format!("Fetching formula: {name}"));
        } else {
            cmd.arg("--quiet");
        }

        match cmd.status().await {
            Ok(status) if status.success() => fetched_formulae.push(name.clone()),
            _ => failed_formulae.push(name.clone()),
        }
    }

    // Fetch casks sequentially
    for name in casks {
        let mut cmd = Command::new("brew");
        cmd.arg("fetch").arg("--cask").arg(name);

        if verbose {
            print_log(LogLevel::Info, &format!("Fetching cask: {name}"));
        } else {
            cmd.arg("--quiet");
        }

        match cmd.status().await {
            Ok(status) if status.success() => fetched_casks.push(name.clone()),
            _ => failed_casks.push(name.clone()),
        }
    }

    FetchedThings {
        formulae: fetched_formulae,
        casks: fetched_casks,
        failed_formulae,
        failed_casks,
    }
}

/// Install formulae/casks sequentially.
/// The argument is a vector of argslices, representing the arguments to the `brew install` subcommand.
async fn install_all(install_tasks: Vec<Vec<String>>) -> anyhow::Result<()> {
    for args in install_tasks {
        let display = format!("brew {}", args.join(" "));
        print_log(LogLevel::Info, &format!("Installing: {display}"));
        let arg_slices: Vec<&str> = args.iter().map(String::as_str).collect();

        let status = Command::new("brew")
            .arg("install")
            .args(&arg_slices)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .stdin(std::process::Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            print_log(LogLevel::Error, &format!("Failed: {display}"));
        }
    }
    Ok(())
}
