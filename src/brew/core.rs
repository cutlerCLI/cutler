// SPDX-License-Identifier: Apache-2.0

use crate::brew::types::{BrewDiff, BrewListType};
use crate::cli::atomic::should_dry_run;
use crate::config::core::Brew;
use crate::util::{
    io::confirm,
    logging::{LogLevel, print_log},
};
use anyhow::{Result, bail};
use nix::NixPath;
use std::{env, path::Path, time::Duration};
use tokio::process::Command;
use tokio::{fs, try_join};

/// Helper for: ensure_brew()
/// Ensures Xcode Command Line Tools are installed.
/// If not, prompts the user to install them (unless dry_run).
async fn ensure_xcode_clt() -> Result<()> {
    async fn check_installed() -> bool {
        let output = Command::new("xcode-select").arg("-p").output().await;
        match output {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                !path.is_empty()
            }
            _ => false,
        }
    }

    // first round check
    // if not, continue to installation process
    let clt_installed = check_installed().await;

    if clt_installed {
        return Ok(());
    }

    if should_dry_run() {
        print_log(
            LogLevel::Dry,
            "Would install Xcode Command Line Tools (not detected)",
        );
        return Ok(());
    }

    print_log(
        LogLevel::Warning,
        "Xcode Command Line Tools are not installed.",
    );

    if confirm("Install Xcode Command Line Tools now?") {
        print_log(
            LogLevel::Info,
            "Waiting to find Xcode Command Line Tools after installation...",
        );
        let status = Command::new("xcode-select")
            .arg("--install")
            .status()
            .await?;

        if !status.success() {
            bail!(
                "Failed to launch Xcode Command Line Tools installer. Try manually installing it using `xcode-select --install`."
            );
        }

        print_log(
            LogLevel::Info,
            "Xcode Command Line Tools installer launched. Waiting for installation to complete...",
        );

        // wait for 60 minutes for the user to finish installation
        // otherwise, bail out
        for _ in 0..720 {
            tokio::time::sleep(Duration::from_millis(5000)).await;

            // loop checks here
            if check_installed().await {
                print_log(LogLevel::Info, "Xcode Command Line Tools installed.");
                return Ok(());
            }
        }

        bail!(
            "Timed out. Re-run this command once installation completes.\nIf there was an error during installation, try running `xcode-select --install` again."
        );
    } else {
        bail!(
            "Xcode Command Line Tools are required for Homebrew operations, but were not found. Aborting."
        );
    }
}

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
    } else if fs::try_exists(Path::new("/usr/local/bin/brew"))
        .await
        .unwrap_or_default()
    {
        let bin = "/usr/local/bin";
        let sbin = "/usr/local/sbin";
        let mut new_path = existing_path.clone();
        if !existing_path.split(':').any(|p| p == bin) {
            new_path = format!("{bin}:{new_path}");
        }
        if !existing_path.split(':').any(|p| p == sbin) {
            new_path = format!("{sbin}:{new_path}");
        }
        unsafe { env::set_var("PATH", &new_path) };
    } else {
        print_log(
            LogLevel::Warning,
            "Brew binary not found in standard directories; PATH not updated.",
        );
    }

    unsafe { env::set_var("HOMEBREW_NO_AUTO_UPDATE", "1") };
    unsafe { env::set_var("HOMEBREW_NO_ANALYTICS", "1") };
    unsafe { env::set_var("HOMEBREW_NO_ENV_HINTS", "1") };

    print_log(
        LogLevel::Info,
        "Homebrew environment configured for this process.",
    );
}

/// Helper for: ensure_brew()
/// Installs Homebrew via the official script.
async fn install_homebrew() -> Result<()> {
    let install_command =
        "curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh | /bin/bash";
    let status = Command::new("/bin/bash")
        .arg("-c")
        .arg(install_command)
        .status()
        .await?;

    print_log(LogLevel::Info, "Installing Homebrew...");

    if !status.success() {
        bail!("Failed to install Homebrew.");
    }

    Ok(())
}

/// Checks if Homebrew is actually installed.
pub async fn is_brew_installed() -> bool {
    !which::which("brew").unwrap_or_default().is_empty()
}

/// Ensures that Homebrew is installed on the machine.
pub async fn ensure_brew() -> Result<()> {
    // ensure xcode command-line tools first
    ensure_xcode_clt().await?;

    if !is_brew_installed().await {
        if should_dry_run() {
            print_log(
                LogLevel::Dry,
                "Would install Homebrew since not found in $PATH",
            );
            return Ok(());
        }

        print_log(LogLevel::Warning, "Homebrew is not installed.");

        if confirm("Install Homebrew now?") {
            install_homebrew().await?;

            // set environment variables for `brew`
            set_homebrew_env_vars().await;

            // re-check that Homebrew is now installed and in $PATH
            let is_installed_after = Command::new("brew")
                .arg("--version")
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !is_installed_after {
                bail!(
                    "Homebrew installation seems to have failed or brew is still not in PATH. Please update your PATH accordingly."
                );
            }
        } else {
            bail!("Homebrew is required for brew operations, but was not found.");
        }
    }

    Ok(())
}

/// Lists Homebrew things (formulae/casks/taps/deps) and separates them based on newline.
pub async fn brew_list(list_type: BrewListType) -> Result<Vec<String>> {
    let args: Vec<String> = match list_type {
        BrewListType::Tap => vec![list_type.to_string()],
        _ => {
            let lt_str = list_type.to_string();
            vec![
                "list".to_string(),
                "--quiet".to_string(),
                "--full-name".to_string(),
                "-1".to_string(),
                lt_str,
            ]
        }
    };

    let output = Command::new("brew").args(&args).output().await?;

    print_log(
        LogLevel::Info,
        &format!("Running {list_type} list command..."),
    );

    if !output.status.success() {
        bail!("Failed to list {list_type}, bailing.")
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Compare the Brew config struct with the actual Homebrew state.
/// Returns a BrewDiff struct with missing/extra formulae, casks, and taps.
pub async fn compare_brew_state(brew_cfg: Brew) -> Result<BrewDiff> {
    let no_deps = brew_cfg.no_deps.unwrap_or(false);

    let config_formulae: Vec<String> = brew_cfg.formulae.clone().unwrap_or_default();
    let config_casks: Vec<String> = brew_cfg.casks.clone().unwrap_or_default();
    let config_taps: Vec<String> = brew_cfg.taps.clone().unwrap_or_default();

    // fetch installed state in parallel
    let (installed_formulae, installed_casks, installed_taps) = try_join!(
        brew_list(BrewListType::Formula),
        brew_list(BrewListType::Cask),
        brew_list(BrewListType::Tap)
    )?;
    let mut installed_formulae = installed_formulae;

    // omit installed as dependency
    if no_deps {
        print_log(LogLevel::Info, "--no-deps used, proceeding with checks...");
        let installed_as_deps = brew_list(BrewListType::Dependency).await?;

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
