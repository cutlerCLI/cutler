use crate::util::io::confirm_action;
use crate::util::logging::{LogLevel, print_log};
use anyhow::Result;
use std::process::Command;

/// Checks if Homebrew is installed on the machine (should be recognizable by $PATH).
pub fn ensure_brew(dry_run: bool) -> Result<()> {
    let is_installed = Command::new("brew")
        .arg("--version")
        .output()
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
            install_homebrew()?
        } else {
            anyhow::bail!("Homebrew is required for brew operations, but was not found.");
        }
    }

    Ok(())
}

/// Installs Homebrew.
fn install_homebrew() -> Result<(), anyhow::Error> {
    let script = "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)";

    print_log(LogLevel::Info, "Installing Homebrew...");

    let status = Command::new("sh").arg("-c").arg(script).status()?;
    if !status.success() {
        anyhow::bail!("Failed to install Homebrew");
    }
    Ok(())
}

/// Lists installed Homebrew formulae / casks.
pub fn brew_list(args: &[&str]) -> Result<Vec<String>> {
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
