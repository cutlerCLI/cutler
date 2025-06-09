use tokio::process::Command;

use crate::{
    brew::utils::{
        brew_list, brew_list_taps, disable_auto_update, ensure_brew, restore_auto_update,
    },
    config::{get_config_path, load_config},
    util::logging::{LogLevel, print_log},
};
use anyhow::{Context, Result};

async fn fetch_all(formulae: &[String], casks: &[String], verbose: bool) {
    let mut handles = Vec::new();

    for name in formulae {
        let name = name.clone();
        handles.push(tokio::spawn(async move {
            let mut cmd = Command::new("brew");
            cmd.arg("fetch").arg(&name);
            if verbose {
                print_log(LogLevel::Info, &format!("Fetching formula: {}", name));
            } else {
                cmd.arg("--quiet");
            }
            let _ = cmd.status().await;
        }));
    }
    for name in casks {
        let name = name.clone();
        handles.push(tokio::spawn(async move {
            let mut cmd = Command::new("brew");
            cmd.arg("fetch").arg("--cask").arg(&name);
            if verbose {
                print_log(LogLevel::Info, &format!("Fetching cask: {}", name));
            } else {
                cmd.arg("--quiet");
            }
            let _ = cmd.status().await;
        }));
    }
    for handle in handles {
        let _ = handle.await;
    }
}

async fn install_sequentially(install_tasks: Vec<Vec<String>>) -> anyhow::Result<()> {
    for args in install_tasks {
        let display = format!("brew {}", args.join(" "));
        print_log(LogLevel::Info, &format!("Installing: {}", display));
        let arg_slices: Vec<&str> = args.iter().map(String::as_str).collect();

        let status = Command::new("brew")
            .args(&arg_slices)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .stdin(std::process::Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            print_log(LogLevel::Error, &format!("Failed: {}", display));
        }
    }
    Ok(())
}

pub async fn run(verbose: bool, dry_run: bool, quiet: bool) -> Result<()> {
    let cfg_path = get_config_path();

    if !cfg_path.exists() {
        if !quiet {
            print_log(
                LogLevel::Error,
                "No config file found. Run `cutler init` to start.",
            );
        }
        return Ok(());
    }

    // disable homebrew auto-update
    let prev = disable_auto_update();

    // ensure homebrew installation
    ensure_brew(dry_run).await?;

    let config = load_config(&cfg_path).await?;
    let brew_cfg = config
        .get("brew")
        .and_then(|i| i.as_table())
        .context("No [brew] table found in config")?;

    // tap all taps listed in the config before fetching/installing, but only if not already tapped
    if let Some(taps_val) = brew_cfg.get("taps").and_then(|v| v.as_array()) {
        let taps: Vec<String> = taps_val
            .iter()
            .filter_map(|x| x.as_str())
            .map(|s| s.to_string())
            .collect();

        // get currently tapped taps
        let tapped_now = brew_list_taps().await.unwrap_or_default();
        for tap in taps {
            if tapped_now.contains(&tap) {
                if verbose && !quiet {
                    print_log(LogLevel::Info, &format!("Already tapped: {}", tap));
                }
                continue;
            }

            if dry_run {
                if !quiet {
                    print_log(LogLevel::Dry, &format!("Would tap {}", tap));
                }
            } else {
                if !quiet {
                    print_log(LogLevel::Info, &format!("Tapping: {}", tap));
                }
                let status = Command::new("brew")
                    .arg("tap")
                    .arg(&tap)
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .stdin(std::process::Stdio::inherit())
                    .status()
                    .await?;
                if !status.success() && !quiet {
                    print_log(LogLevel::Error, &format!("Failed to tap: {}", tap));
                }
            }
        }
    }

    // fetch currently installed items to skip those
    let installed_formulas = brew_list(&["list", "--formula"]).await?;
    let installed_casks = brew_list(&["list", "--cask"]).await?;

    // warn about extra installed formulae not in config
    let config_formulae = brew_cfg
        .get("formulae")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let extra_formulae: Vec<_> = installed_formulas
        .iter()
        .filter(|f| !config_formulae.contains(f))
        .collect();

    if !extra_formulae.is_empty() && !quiet {
        print_log(
            LogLevel::Warning,
            &format!(
                "Extra installed formulae not in config: {:?}",
                extra_formulae
            ),
        );
    }

    // warn about extra installed casks not in config
    let config_casks = brew_cfg
        .get("casks")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let extra_casks: Vec<_> = installed_casks
        .iter()
        .filter(|c| !config_casks.contains(c))
        .collect();

    if !extra_casks.is_empty() && !quiet {
        print_log(
            LogLevel::Warning,
            &format!("Extra installed casks not in config: {:?}", extra_casks),
        );
    }

    // extra message
    if (!extra_formulae.is_empty() || !extra_casks.is_empty()) && !quiet {
        println!("\nRun `cutler brew backup` to synchronize your config with the system");
    }

    // collect all install tasks, skipping already installed
    let mut install_tasks: Vec<Vec<String>> = Vec::new();
    let mut to_fetch_formulae: Vec<String> = Vec::new();
    let mut to_fetch_casks: Vec<String> = Vec::new();

    if let Some(arr) = brew_cfg.get("formulae").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(name) = v.as_str() {
                if installed_formulas.contains(&name.to_string()) {
                    if verbose && !quiet {
                        print_log(
                            LogLevel::Info,
                            &format!("Skipping already installed formula: {}", name),
                        );
                    }
                } else {
                    install_tasks.push(vec!["install".to_string(), name.to_string()]);
                    to_fetch_formulae.push(name.to_string());
                }
            }
        }
    }
    if let Some(arr) = brew_cfg.get("casks").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(name) = v.as_str() {
                if installed_casks.contains(&name.to_string()) {
                    if verbose && !quiet {
                        print_log(
                            LogLevel::Info,
                            &format!("Skipping already installed cask: {}", name),
                        );
                    }
                } else {
                    install_tasks.push(vec![
                        "install".to_string(),
                        "--cask".to_string(),
                        name.to_string(),
                    ]);
                    to_fetch_casks.push(name.to_string());
                }
            }
        }
    }

    if dry_run {
        for args in &install_tasks {
            let display = format!("brew {}", args.join(" "));
            if !quiet {
                print_log(LogLevel::Dry, &display);
            }
        }
    } else {
        // pre-download everything in parallel
        if (!to_fetch_formulae.is_empty() || !to_fetch_casks.is_empty()) && !quiet {
            print_log(LogLevel::Info, "Pre-downloading all formulae and casks...");
        }
        fetch_all(&to_fetch_formulae, &to_fetch_casks, verbose && !quiet).await;

        // sequentially install
        install_sequentially(install_tasks).await?;
    }

    restore_auto_update(prev);
    Ok(())
}
