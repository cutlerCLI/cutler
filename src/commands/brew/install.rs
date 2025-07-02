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
                    LogLevel::Warning,
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

        let (fetched_formulae, fetched_casks) =
            fetch_all(&brew_diff.missing_formulae, &brew_diff.missing_casks).await;

        // build install tasks only for successfully fetched items
        let mut install_tasks: Vec<Vec<String>> = Vec::new();

        for name in fetched_formulae {
            install_tasks.push(vec!["install".to_string(), "--formula".to_string(), name]);
        }
        for name in fetched_casks {
            install_tasks.push(vec!["install".to_string(), "--cask".to_string(), name]);
        }

        // sequentially install only the successfully fetched items
        install_sequentially(install_tasks).await?;

        Ok(())
    }
}

/// Downloads all formulae/casks before installation.
/// Returns only the successfully fetched formulae and casks.
async fn fetch_all(formulae: &[String], casks: &[String]) -> (Vec<String>, Vec<String>) {
    let mut handles = Vec::new();

    for name in formulae {
        let name = name.clone();
        handles.push(tokio::spawn(async move {
            let mut cmd = Command::new("brew");
            cmd.arg("fetch").arg(&name);

            if is_verbose() {
                print_log(LogLevel::Info, &format!("Fetching formula: {name}"));
            } else {
                cmd.arg("--quiet");
            }

            match cmd.status().await {
                Ok(status) if status.success() => Some(("formula".to_string(), name)),
                _ => None,
            }
        }));
    }
    for name in casks {
        let name = name.clone();
        handles.push(tokio::spawn(async move {
            let mut cmd = Command::new("brew");
            cmd.arg("fetch").arg("--cask").arg(&name);

            if is_verbose() {
                print_log(LogLevel::Info, &format!("Fetching cask: {name}"));
            } else {
                cmd.arg("--quiet");
            }

            match cmd.status().await {
                Ok(status) if status.success() => Some(("cask".to_string(), name)),
                _ => None,
            }
        }));
    }

    let mut fetched_formulae = Vec::new();
    let mut fetched_casks = Vec::new();

    for handle in handles {
        if let Ok(Some((item_type, name))) = handle.await {
            match item_type.as_str() {
                "formula" => fetched_formulae.push(name),
                "cask" => fetched_casks.push(name),
                _ => {}
            }
        }
    }

    (fetched_formulae, fetched_casks)
}

/// Install formulae/casks sequentially.
/// The argument is a vector of vectors of strings, with each vector representing arguments to a brew command.
async fn install_sequentially(install_tasks: Vec<Vec<String>>) -> anyhow::Result<()> {
    for args in install_tasks {
        let display = format!("brew {}", args.join(" "));
        print_log(LogLevel::Info, &format!("Installing: {display}"));
        let arg_slices: Vec<&str> = args.iter().map(String::as_str).collect();

        let status = Command::new("brew")
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
