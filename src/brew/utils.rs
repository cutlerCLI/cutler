use crate::util::io::confirm_action;
use crate::util::logging::{LogLevel, print_log};
use anyhow::Result;
use std::env;
use tokio::process::Command;

/// Ensures Xcode Command Line Tools are installed.
/// If not, prompts the user to install them (unless dry_run).
pub async fn ensure_xcode_clt(dry_run: bool) -> Result<()> {
    let output = Command::new("xcode-select").arg("-p").output().await;

    let clt_installed = match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            !path.is_empty()
        }
        _ => false,
    };

    if clt_installed {
        return Ok(());
    }

    if dry_run {
        print_log(
            LogLevel::Info,
            "Would install Xcode Command Line Tools (not detected)",
        );
        return Ok(());
    }

    print_log(
        LogLevel::Warning,
        "Xcode Command Line Tools are not installed.",
    );

    if confirm_action("Install Xcode Command Line Tools now?")? {
        print_log(LogLevel::Info, "Installing Xcode Command Line Tools...");
        let status = Command::new("xcode-select")
            .arg("--install")
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("Failed to launch Xcode Command Line Tools installer.");
        }
        print_log(
            LogLevel::Info,
            "Xcode Command Line Tools installer launched. Please complete installation before continuing.",
        );
        // Wait for user to finish installation
        // Optionally, could poll for completion, but for now, just bail out.
        anyhow::bail!(
            "Xcode Command Line Tools installation required. Please re-run the command after installation completes."
        );
    } else {
        anyhow::bail!(
            "Xcode Command Line Tools are required for Homebrew operations, but were not found."
        );
    }
}

/// Checks if Homebrew is installed on the machine (should be recognizable by $PATH).
pub async fn ensure_brew(dry_run: bool) -> Result<()> {
    // ensure xcode command-line tools first
    ensure_xcode_clt(dry_run).await?;

    let is_installed = Command::new("brew")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_installed {
        if dry_run {
            print_log(
                LogLevel::Info,
                "Would install Homebrew since not found in $PATH",
            );
            return Ok(());
        }

        print_log(LogLevel::Warning, "Homebrew is not installed.");

        if confirm_action("Install Homebrew now?")? {
            install_homebrew().await?
        } else {
            anyhow::bail!("Homebrew is required for brew operations, but was not found.");
        }
    }

    Ok(())
}

/// Lists all currently tapped Homebrew taps.
pub async fn brew_list_taps() -> Result<Vec<String>> {
    let output = Command::new("brew").arg("tap").output().await?;
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

/// Installs Homebrew.
async fn install_homebrew() -> Result<(), anyhow::Error> {
    let script = "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)";

    print_log(LogLevel::Info, "Installing Homebrew...");

    let status = Command::new("sh").arg("-c").arg(script).status().await?;
    if !status.success() {
        anyhow::bail!("Failed to install Homebrew");
    }
    Ok(())
}

/// Lists installed Homebrew formulae / casks.
pub async fn brew_list(args: &[&str]) -> Result<Vec<String>> {
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

/// Disables Homebrew auto-update globally for the process, returning previous value.
/// Call this before brew commands.
pub fn disable_auto_update() -> Option<String> {
    let prev = env::var("HOMEBREW_NO_AUTO_UPDATE").ok();
    unsafe { env::set_var("HOMEBREW_NO_AUTO_UPDATE", "1") };
    prev
}

/// Restores Homebrew auto-update to the given previous value.
/// Call this after brew commands.
pub fn restore_auto_update(prev: Option<String>) {
    unsafe {
        match prev {
            Some(v) => env::set_var("HOMEBREW_NO_AUTO_UPDATE", v),
            None => env::remove_var("HOMEBREW_NO_AUTO_UPDATE"),
        }
    }
}

/// Struct representing the diff between config and installed Homebrew state.
#[derive(Debug, Default)]
pub struct BrewDiff {
    pub missing_formulae: Vec<String>,
    pub extra_formulae: Vec<String>,
    pub missing_casks: Vec<String>,
    pub extra_casks: Vec<String>,
    pub missing_taps: Vec<String>,
    pub extra_taps: Vec<String>,
}

/// Compare the [brew] config table with the actual Homebrew state.
/// Returns a BrewDiff struct with missing/extra formulae, casks, and taps.
/// `brew_cfg` should be a reference to the [brew] table as toml::value::Table.
pub async fn compare_brew_state(brew_cfg: &toml::value::Table) -> Result<BrewDiff> {
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
    let installed_formulae = brew_list(&["list", "--formula"]).await?;
    let installed_casks = brew_list(&["list", "--cask"]).await?;
    let installed_taps = brew_list_taps().await?;

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
