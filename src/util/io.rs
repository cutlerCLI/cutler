use dialoguer::Confirm;
use tokio::process::Command;

use crate::util::{
    globals::{should_accept_interactive, should_dry_run, should_not_restart_services},
    logging::{LogLevel, print_log},
};
use anyhow::Result;

/// Ask "Y/N?"; returns true if accept_interactive is set or the user types "y" or "Y"
pub fn confirm_action(prompt: &str) -> Result<bool> {
    if should_accept_interactive() {
        print_log(
            LogLevel::Prompt,
            &format!("{} [y/N]: y (auto-accepted)", prompt),
        );
        return Ok(true);
    }

    let result = Confirm::new().with_prompt(prompt).interact()?;
    Ok(result)
}

/// Restart Finder, Dock, SystemUIServer so defaults take effect.
pub async fn restart_system_services_if_needed() -> Result<(), anyhow::Error> {
    if should_not_restart_services() {
        return Ok(());
    }

    let dry_run = should_dry_run();

    // services to restart
    const SERVICES: &[&str] = &["cfprefsd", "SystemUIServer", "Dock", "Finder"];

    for svc in SERVICES {
        if dry_run {
            print_log(LogLevel::Dry, &format!("Would restart {}", svc));
        } else {
            let out = Command::new("killall").arg(svc).output().await?;
            if !out.status.success() {
                print_log(LogLevel::Error, &format!("Failed to restart {}", svc));
            } else {
                print_log(LogLevel::Info, &format!("{} restarted", svc));
            }
        }
    }
    if !dry_run {
        print_log(
            LogLevel::Fruitful,
            "Done. Log out and log back in to allow your Mac some time!",
        );
    } else {
        print_log(LogLevel::Dry, "Would restart system services.");
    }
    Ok(())
}
