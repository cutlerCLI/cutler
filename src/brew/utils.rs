use crate::brew::types::{BrewDiff, BrewListType};
use crate::util::{
    globals::should_dry_run,
    io::confirm_action,
    logging::{LogLevel, print_log},
};
use anyhow::Result;
use std::env;
use std::time::Duration;
use tokio::process::Command;

/// Helper for: ensure_brew()
/// Ensures Xcode Command Line Tools are installed.
/// If not, prompts the user to install them (unless dry_run).
async fn ensure_xcode_clt() -> Result<()> {
    async fn check_installed() -> Result<bool> {
        let output = Command::new("xcode-select").arg("-p").output().await;

        let clt_installed = match output {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                !path.is_empty()
            }
            _ => false,
        };

        Ok(clt_installed)
    }

    let clt_installed = check_installed().await?;

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

    if confirm_action("Install Xcode Command Line Tools now?")? {
        print_log(
            LogLevel::Info,
            "Waiting to find Xcode Command Line Tools after installation...",
        );
        let status = Command::new("xcode-select")
            .arg("--install")
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!(
                "Failed to launch Xcode Command Line Tools installer. Try manually installing it using `xcode-select --install`."
            );
        }

        print_log(
            LogLevel::Info,
            "Xcode Command Line Tools installer launched. Waiting for installation to complete...",
        );

        // wait for 20 minutes for the user to finish installation
        // otherwise, bail out
        for _ in 0..240 {
            tokio::time::sleep(Duration::from_millis(5000)).await;

            if check_installed().await.unwrap() {
                print_log(LogLevel::Info, "Xcode Command Line Tools installed.");
                return Ok(());
            }
        }

        anyhow::bail!("Timed out. Re-run this command after installation completes.");
    } else {
        anyhow::bail!(
            "Xcode Command Line Tools are required for Homebrew operations, but were not found. Aborting."
        );
    }
}

/// Helper for: ensure_brew()
/// Installs Homebrew.
async fn install_homebrew() -> Result<(), anyhow::Error> {
    let script = "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)";

    print_log(LogLevel::Info, "Installing Homebrew...");

    let status = Command::new("sh").arg("-c").arg(script).status().await?;
    if !status.success() {
        anyhow::bail!("Failed to install Homebrew.");
    }
    Ok(())
}

/// Checks if Homebrew is installed on the machine (should be recognizable by $PATH).
pub async fn ensure_brew() -> Result<()> {
    // ensure xcode command-line tools first
    ensure_xcode_clt().await?;

    let is_installed = Command::new("brew")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_installed {
        if should_dry_run() {
            print_log(
                LogLevel::Dry,
                "Would install Homebrew since not found in $PATH",
            );
            return Ok(());
        }

        print_log(LogLevel::Warning, "Homebrew is not installed.");

        if confirm_action("Install Homebrew now?")? {
            install_homebrew().await?;

            // update PATH with brew binary location
            let existing_path = std::env::var("PATH").unwrap_or_default();
            if std::path::Path::new("/opt/homebrew/bin/brew").exists() {
                let new_path = format!("/opt/homebrew/bin:{existing_path}");
                unsafe { std::env::set_var("PATH", &new_path) };
                print_log(LogLevel::Info, "Updated PATH with /opt/homebrew/bin");
            } else if std::path::Path::new("/usr/local/bin/brew").exists() {
                let new_path = format!("/usr/local/bin:{existing_path}");
                unsafe { std::env::set_var("PATH", &new_path) };
                print_log(LogLevel::Info, "Updated PATH with /usr/local/bin");
            } else {
                print_log(
                    LogLevel::Warning,
                    "Brew binary not found in standard directories; PATH not updated.",
                );
            }

            // re-check that Homebrew is now installed and in $PATH
            let is_installed_after = Command::new("brew")
                .arg("--version")
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !is_installed_after {
                anyhow::bail!(
                    "Homebrew installation seems to have failed or brew is still not in PATH. Please update your PATH accordingly."
                );
            }
        } else {
            anyhow::bail!("Homebrew is required for brew operations, but was not found.");
        }
    }

    Ok(())
}

/// Lists Homebrew things (formulae/casks/taps/deps) and separates them based on newline.
pub async fn brew_list(list_type: BrewListType) -> Result<Vec<String>> {
    let args = if list_type == BrewListType::Cask {
        vec!["list", "--casks"]
    } else if list_type == BrewListType::Formula {
        vec!["list", "--formulae"]
    } else if list_type == BrewListType::Tap {
        vec!["tap"]
    } else {
        vec!["list", "--installed-as-dependency"]
    };

    let output = Command::new("brew").args(args).output().await?;

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

/// Disables Homebrew auto-update globally for the process.
pub fn disable_brew_auto_update() {
    unsafe { env::set_var("HOMEBREW_NO_AUTO_UPDATE", "1") };
}

/// Compare the [brew] config table with the actual Homebrew state.
/// Returns a BrewDiff struct with missing/extra formulae, casks, and taps.
/// `brew_cfg` should be a reference to the [brew] table as toml::value::Table.
pub async fn compare_brew_state(brew_cfg: &toml::value::Table) -> Result<BrewDiff> {
    print_log(
        LogLevel::Info,
        "Starting comparison of Homebrew state with config...",
    );

    let no_deps = brew_cfg
        .get("no-deps")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let config_formulae: Vec<String> = brew_cfg
        .get("formulae")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let config_casks: Vec<String> = brew_cfg
        .get("casks")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let config_taps: Vec<String> = brew_cfg
        .get("taps")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    // fetch installed state
    let mut installed_formulae = brew_list(BrewListType::Formula).await?;
    let installed_casks = brew_list(BrewListType::Cask).await?;
    let installed_taps = brew_list(BrewListType::Tap).await?;

    // omit installed as dependency
    if no_deps {
        print_log(LogLevel::Info, "deps-check found to be true, proceeding...");
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
