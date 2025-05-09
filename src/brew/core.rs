use crate::config::loader::load_config;
use crate::util::logging::{LogLevel, print_log};
use anyhow::{Context, Result};
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::{env, path::PathBuf};
use std::{fs, process::Command};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

/// Checks if Homebrew is installed on the machine (should be recognizable by $PATH).
pub fn is_brew_installed() -> bool {
    Command::new("brew")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Installs Homebrew.
pub fn install_homebrew(dry_run: bool) -> Result<()> {
    let script = "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)";
    if dry_run {
        print_log(
            LogLevel::Info,
            &format!("Dry-run: would install Homebrew using: {}", script),
        );
        return Ok(());
    }
    print_log(LogLevel::Info, "Installing Homebrew...");
    let status = Command::new("bash").arg("-c").arg(script).status()?;
    if !status.success() {
        anyhow::bail!("Failed to install Homebrew");
    }
    Ok(())
}

/// Lists installed Homebrew formulae / casks.
fn brew_list(args: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("brew").args(args).output()?;
    if !output.status.success() {
        return Ok(vec![]);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Backs up the list of the currently installed Homebrew formulae / casks into the user's config file.
pub fn backup(cfg_path: &std::path::Path, verbose: bool, dry_run: bool) -> Result<()> {
    let formulas = brew_list(&["list", "--formula"])?;
    let casks = brew_list(&["list", "--cask"])?;
    if dry_run {
        print_log(
            LogLevel::Info,
            &format!(
                "Dry-run: would backup {} formulae and {} casks",
                formulas.len(),
                casks.len()
            ),
        );
        return Ok(());
    }
    let mut doc = if cfg_path.exists() {
        let text = fs::read_to_string(cfg_path)?;
        text.parse::<DocumentMut>()
            .context("Failed to parse config TOML")?
    } else {
        DocumentMut::new()
    };
    let brew_item = doc.entry("brew").or_insert(Item::Table(Table::new()));
    let brew_tbl = brew_item.as_table_mut().unwrap();

    // build TOML arrays for formulae and casks
    let mut formula_arr = Array::new();
    for formula in &formulas {
        formula_arr.push(formula.as_str());
    }
    brew_tbl["formulae"] = Item::Value(Value::Array(formula_arr));

    let mut cask_arr = Array::new();
    for cask in &casks {
        cask_arr.push(cask.as_str());
    }
    brew_tbl["casks"] = Item::Value(Value::Array(cask_arr));

    // give length of both lists in verbose, and let the user know about config location
    if verbose {
        print_log(
            LogLevel::Info,
            &format!(
                "Pushed {} casks and {} formulae.",
                formulas.len(),
                casks.len(),
            ),
        );
        print_log(LogLevel::Info, &format!("Writing backup to {:?}", cfg_path));
    }

    fs::write(cfg_path, doc.to_string())?;

    if verbose {
        print_log(
            LogLevel::Success,
            &format!("Backup saved to {:?}", cfg_path),
        );
    } else {
        println!(
            "\n🍎 Done! You can find the backup in your config file location {:?}",
            cfg_path
        );
    }

    Ok(())
}

/// Receives a PathBuf to the user's current configuration and reads it to install Homebrew formulae / casks.
pub fn install_from_config(cfg_path: PathBuf, verbose: bool, dry_run: bool) -> Result<()> {
    let config = load_config(&cfg_path)?;
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
