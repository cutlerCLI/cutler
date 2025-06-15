use dialoguer::Confirm;
use tokio::process::Command;

use crate::{
    commands::GlobalArgs,
    util::{
        globals::should_accept_interactive,
        logging::{LogLevel, print_log},
    },
};
use anyhow::Result;

/// Ask "Y/N?"; returns true if accept_interactive is set or the user types "y" or "Y"
pub fn confirm_action(prompt: &str) -> Result<bool> {
    if should_accept_interactive() {
        print_log(
            LogLevel::Info,
            &format!("{} [y/N]: y (auto-accepted)", prompt),
        );
        return Ok(true);
    }

    let result = Confirm::new().with_prompt(prompt).interact()?;
    Ok(result)
}

/// Restart Finder, Dock, SystemUIServer so defaults take effect.
pub async fn restart_system_services(g: &GlobalArgs) -> Result<(), anyhow::Error> {
    let verbose = g.verbose;
    let dry_run = g.dry_run;

    // services to restart
    const SERVICES: &[&str] = &["cfprefsd", "SystemUIServer", "Dock", "Finder"];

    for svc in SERVICES {
        if dry_run {
            if verbose {
                print_log(LogLevel::Dry, &format!("Would restart {}", svc));
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
        print_log(
            LogLevel::Fruitful,
            "Done. Log out and log back in to allow your Mac some time!",
        );
    } else if dry_run {
        print_log(LogLevel::Dry, "Would restart system services.");
    }
    Ok(())
}
