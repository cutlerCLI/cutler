use dialoguer::Confirm;
use tokio::process::Command;

use crate::util::logging::{LogLevel, print_log};
use anyhow::Result;

/// Global flag to automatically accept all prompts
static mut ACCEPT_ALL: bool = false;

/// Set the global accept_all flag
pub fn set_accept_all(value: bool) {
    unsafe { ACCEPT_ALL = value }
}

/// Ask "Y/N?"; returns true if accept_all is set or the user types "y" or "Y"
pub fn confirm_action(prompt: &str) -> Result<bool> {
    unsafe {
        if ACCEPT_ALL {
            println!("{} [y/N]: y (auto-accepted)", prompt);
            return Ok(true);
        }
    }

    let result = Confirm::new().with_prompt(prompt).interact()?;
    Ok(result)
}

/// Restart Finder, Dock, SystemUIServer so defaults take effect.
pub async fn restart_system_services(verbose: bool, dry_run: bool) -> Result<(), anyhow::Error> {
    const SERVICES: &[&str] = &["Finder", "Dock", "SystemUIServer"];
    for svc in SERVICES {
        if dry_run {
            if verbose {
                print_log(LogLevel::Info, &format!("Would restart {}", svc));
            }
        } else {
            let out = Command::new("killall").arg(svc).output().await?;
            if !out.status.success() {
                print_log(LogLevel::Error, &format!("Failed to restart {}", svc));
            } else if verbose {
                print_log(LogLevel::Success, &format!("{} restarted", svc));
            }
        }
    }
    if !verbose && !dry_run {
        println!("\n🍎 Done. System services restarted.");
    } else if dry_run {
        println!("\n🍎 Dry‑run: would restart services.");
    }
    Ok(())
}
