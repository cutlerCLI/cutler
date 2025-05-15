use std::env;
use tokio::process::Command;

use crate::{
    brew::utils::{brew_list, ensure_brew},
    config::{get_config_path, load_config},
    util::logging::{LogLevel, print_log},
};
use anyhow::{Context, Result};

pub async fn run(verbose: bool, dry_run: bool) -> Result<()> {
    let cfg_path = &get_config_path();

    if !cfg_path.exists() {
        print_log(
            LogLevel::Error,
            "No config file found. Run `cutler init` to start.",
        );
        return Ok(());
    }

    // ensure homebrew installation
    ensure_brew(dry_run)?;

    let config = load_config(cfg_path).await?;
    let brew_cfg = config
        .get("brew")
        .and_then(|i| i.as_table())
        .context("No [brew] table found in config")?;

    // disables automatic upgrades since cutler shouldn't be used for that
    // brew upgrade --greedy exists
    const ENV_VAR: &str = "HOMEBREW_NO_INSTALL_UPGRADE";

    let old_value = env::var(ENV_VAR).ok();
    unsafe {
        env::set_var(ENV_VAR, "1");

        if verbose {
            print_log(LogLevel::Info, &format!("Setting {} to 1", ENV_VAR));
        }
    }

    // fetch currently installed items to skip those
    let installed_formulas = brew_list(&["list", "--formula"])?;
    let installed_casks = brew_list(&["list", "--cask"])?;

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

    if !extra_formulae.is_empty() {
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

    if !extra_casks.is_empty() {
        print_log(
            LogLevel::Warning,
            &format!("Extra installed casks not in config: {:?}", extra_casks),
        );
    }

    // extra message
    if !extra_formulae.is_empty() || !extra_casks.is_empty() {
        println!("\nRun `cutler brew backup` to synchronize your config with the system");
    }

    // collect all install tasks, skipping already installed
    let mut install_tasks: Vec<Vec<String>> = Vec::new();
    if let Some(arr) = brew_cfg.get("formulae").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(name) = v.as_str() {
                if installed_formulas.contains(&name.to_string()) {
                    if verbose {
                        print_log(
                            LogLevel::Info,
                            &format!("Skipping already installed formula: {}", name),
                        );
                    }
                } else {
                    install_tasks.push(vec!["install".to_string(), name.to_string()]);
                }
            }
        }
    }
    if let Some(arr) = brew_cfg.get("casks").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(name) = v.as_str() {
                if installed_casks.contains(&name.to_string()) {
                    if verbose {
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
                }
            }
        }
    }

    if dry_run {
        for args in &install_tasks {
            let display = format!("brew {}", args.join(" "));
            print_log(LogLevel::Info, &format!("Dry-run: {}", display));
        }
    } else {
        // execute installs concurrently
        let mut handles = Vec::new();
        for args in install_tasks {
            handles.push(tokio::spawn(async move {
                let display = format!("brew {}", args.join(" "));
                if verbose {
                    print_log(LogLevel::Info, &display);
                }
                let arg_slices: Vec<&str> = args.iter().map(String::as_str).collect();
                match Command::new("brew").args(&arg_slices).status().await {
                    Ok(status) if !status.success() => {
                        print_log(LogLevel::Error, &format!("Failed: {}", display));
                    }
                    Err(e) => {
                        print_log(
                            LogLevel::Error,
                            &format!("Error running brew {}: {}", display, e),
                        );
                    }
                    _ => {}
                }
            }));
        }
        for handle in handles {
            let _ = handle.await;
        }
    }

    // restore or unset the environment variable
    if let Some(prev) = old_value {
        unsafe { env::set_var(ENV_VAR, prev.clone()) }
        if verbose {
            print_log(
                LogLevel::Info,
                &format!("Restored {} to previous value: {}", ENV_VAR, prev),
            );
        }
    } else {
        unsafe { env::remove_var(ENV_VAR) };
        if verbose {
            print_log(LogLevel::Info, &format!("Unset {}", ENV_VAR));
        }
    }

    Ok(())
}
