use std::{env, process::Command};

use crate::{
    brew::utils::{brew_list, ensure_brew},
    config::{get_config_path, load_config},
    util::logging::{LogLevel, print_log},
};
use anyhow::{Context, Result};
use rayon::{ThreadPoolBuilder, prelude::*};

pub fn run(verbose: bool, dry_run: bool) -> Result<()> {
    let cfg_path = &get_config_path();

    if !cfg_path.exists() {
        print_log(
            LogLevel::Error,
            "No config file found. Run `cutler init` to start.",
        );
    }

    // ensure homebrew installation
    ensure_brew(dry_run)?;

    let config = load_config(cfg_path)?;
    let brew_cfg = config
        .get("brew")
        .and_then(|i| i.as_table())
        .context("No [brew] table found in config")?;

    // disables automatic upgrades since cutler shouldn't be used for that
    // brew upgrade --greedy exists
    const ENV_VAR: &str = "HOMEBREW_NO_INSTALL_UPGRADE";

    let old_value = env::var(ENV_VAR).ok();
    unsafe { env::set_var(ENV_VAR, "1") }

    // fetch currently installed items to skip those
    let installed_formulas = brew_list(&["list", "--formula"])?;
    let installed_casks = brew_list(&["list", "--cask"])?;

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
        // execute up to 5 installs in parallel
        let pool = ThreadPoolBuilder::new().build()?;

        pool.install(|| {
            install_tasks.par_iter().for_each(|args| {
                let display = format!("brew {}", args.join(" "));

                if verbose {
                    print_log(LogLevel::Info, &display);
                }
                let arg_slices: Vec<&str> = args.iter().map(String::as_str).collect();

                match Command::new("brew").args(&arg_slices).status() {
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
            });
        });
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
