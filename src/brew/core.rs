// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::brew::types::{BrewDiff, BrewListType};
use crate::brew::xcode::ensure_xcode_clt;
use crate::cli::atomic::should_dry_run;
use crate::config::core::Brew;
use crate::util::io::confirm;
use crate::{log_dry, log_info, log_warn};
use anyhow::{Result, bail};
use std::{env, path::Path};
use tokio::process::Command;
use tokio::{fs, try_join};

/// Sets the required environment variables for cutler to interact with Homebrew.
async fn set_homebrew_env_vars() {
    let existing_path = std::env::var("PATH").unwrap_or_default();

    if fs::try_exists(Path::new("/opt/homebrew/bin/brew"))
        .await
        .unwrap_or_default()
    {
        let bin = "/opt/homebrew/bin";
        let sbin = "/opt/homebrew/sbin";
        let mut new_path = existing_path.clone();
        if !existing_path.split(':').any(|p| p == bin) {
            new_path = format!("{bin}:{new_path}");
        }
        if !existing_path.split(':').any(|p| p == sbin) {
            new_path = format!("{sbin}:{new_path}");
        }
        unsafe { env::set_var("PATH", &new_path) };
    } else {
        log_warn!("Brew binary not found in standard directories; $PATH not updated.");
    }

    unsafe { env::set_var("HOMEBREW_NO_AUTO_UPDATE", "1") };
    unsafe { env::set_var("HOMEBREW_NO_ANALYTICS", "1") };
    unsafe { env::set_var("HOMEBREW_NO_ENV_HINTS", "1") };

    log_info!("Homebrew environment has been configured for this process.");
}

/// Helper for: `ensure_brew()`
/// Installs Homebrew via the official script.
async fn install_homebrew() -> Result<()> {
    let install_command =
        "curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh | /bin/bash";

    let status = Command::new("/bin/bash")
        .arg("-c")
        .arg(install_command)
        .status()
        .await?;

    log_info!("Installing Homebrew...");

    if !status.success() {
        bail!("Homebrew install script failed: {status}");
    }

    Ok(())
}

/// Checks if Homebrew is actually installed.
pub async fn brew_is_installed() -> bool {
    Command::new("brew")
        .arg("--version")
        .output()
        .await
        .map(|op| op.status.success())
        .unwrap_or(false)
}

/// Ensures that Homebrew is installed on the machine.
pub async fn ensure_brew() -> Result<()> {
    // ensure xcode command-line tools first
    ensure_xcode_clt().await?;

    if !brew_is_installed().await {
        if should_dry_run() {
            log_dry!("Would install Homebrew since not found in $PATH.");

            return Ok(());
        }
        log_warn!("Homebrew is not installed.");

        if confirm("Install Homebrew now?") {
            install_homebrew().await?;

            // set environment variables for `brew`
            set_homebrew_env_vars().await;

            if !brew_is_installed().await {
                bail!("Homebrew installation seems to have failed or brew is still not in $PATH.");
            }
        } else {
            bail!("Homebrew is required for brew operations, but was not found.");
        }
    }

    Ok(())
}

/// Flattens tap prefixes for a given list of strings.
///
/// `vec!["some/cool/program", "other_program"]` -> `vec!["some/cool/program", "program", "other_program"]`
fn flatten_tap_prefix(lines: Vec<String>) -> Vec<String> {
    lines
        .iter()
        .flat_map(|l| {
            let parts: Vec<&str> = l.split('/').collect();
            if parts.len() == 3 {
                vec![l.clone(), parts[2].to_string()]
            } else {
                vec![l.clone()]
            }
        })
        .collect()
}

/// Lists Homebrew things (formulae/casks/taps/deps) and separates them based on newline.
/// Note that `flatten` will be ignored if `list_type` is `BrewListType::Tap`.
pub async fn brew_list(list_type: BrewListType, flatten: bool) -> Result<Vec<String>> {
    let args: Vec<String> = if list_type == BrewListType::Tap { vec![list_type.to_string()] } else {
        let lt_str = list_type.to_string();
        vec![
            "list".to_string(),
            "--quiet".to_string(),
            "--full-name".to_string(),
            "-1".to_string(),
            lt_str,
        ]
    };

    let output = Command::new("brew").args(&args).output().await?;
    log_info!("Running {list_type} list command...");

    if !output.status.success() {
        log_warn!("{list_type} listing failed, will return empty.");
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<String> = stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if flatten {
        lines = flatten_tap_prefix(lines);
    }

    Ok(lines)
}

/// Compare the Brew config struct with the actual Homebrew state.
/// Returns a `BrewDiff` struct with missing/extra formulae, casks, and taps.
pub async fn diff_brew(brew_cfg: Brew) -> Result<BrewDiff> {
    let no_deps = brew_cfg.no_deps.unwrap_or(false);

    let config_formulae: Vec<String> =
        flatten_tap_prefix(brew_cfg.formulae.clone().unwrap_or_default());
    let config_casks: Vec<String> = flatten_tap_prefix(brew_cfg.casks.clone().unwrap_or_default());
    let config_taps: Vec<String> = brew_cfg.taps.clone().unwrap_or_default();

    // fetch installed state in parallel
    let (mut installed_formulae, installed_casks, installed_taps) = try_join!(
        brew_list(BrewListType::Formula, true),
        brew_list(BrewListType::Cask, true),
        brew_list(BrewListType::Tap, false) // no need for flattening here
    )?;

    // omit installed as dependency
    if no_deps {
        log_info!("--no-deps used, proceeding with checks...");
        let installed_as_deps = brew_list(BrewListType::Dependency, true).await?;

        installed_formulae = installed_formulae
            .iter()
            .filter(|f| !installed_as_deps.contains(f))
            .cloned()
            .collect();
    }

    // compute missing/extra
    let missing_formulae: Vec<String> = config_formulae
        .iter()
        .filter(|f| !installed_formulae.contains(f))
        .cloned()
        .collect();
    let extra_formulae: Vec<String> = installed_formulae
        .iter()
        .filter(|f| !config_formulae.contains(f))
        .cloned()
        .collect();

    let missing_casks: Vec<String> = config_casks
        .iter()
        .filter(|c| !installed_casks.contains(c))
        .cloned()
        .collect();
    let extra_casks: Vec<String> = installed_casks
        .iter()
        .filter(|c| !config_casks.contains(c))
        .cloned()
        .collect();

    let missing_taps: Vec<String> = config_taps
        .iter()
        .filter(|t| !installed_taps.contains(t))
        .cloned()
        .collect();
    let extra_taps: Vec<String> = installed_taps
        .iter()
        .filter(|t| !config_taps.contains(t))
        .cloned()
        .collect();

    Ok(BrewDiff {
        missing_formulae,
        extra_formulae,
        missing_casks,
        extra_casks,
        missing_taps,
        extra_taps,
    })
}
