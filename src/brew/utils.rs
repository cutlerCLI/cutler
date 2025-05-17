use crate::util::io::confirm_action;
use crate::util::logging::{LogLevel, print_log};
use anyhow::Result;
use std::env;
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

/// Checks if a formula is a dependency of other formulae.
pub fn is_dependency(formula: &str) -> bool {
    let output = Command::new("brew")
        .args(["uses", "--installed", formula])
        .output()
        .unwrap();

    !output.stdout.is_empty()
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
