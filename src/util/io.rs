// SPDX-License-Identifier: Apache-2.0

use dialoguer::Confirm;
use mac_notification_sys::*;
use tokio::process::Command;

use crate::{
    cli::atomic::{should_accept_all, should_dry_run, should_not_restart_services},
    util::logging::{LogLevel, print_log},
};
use anyhow::Result;

/// Ask "Y/N?"; returns true if accept_all is set or the user types "y" or "Y"
pub fn confirm_action(prompt: &str) -> bool {
    if should_accept_all() {
        print_log(
            LogLevel::Prompt,
            &format!("{prompt} [y/N]: y (auto-accepted)"),
        );

        return true;
    }

    Confirm::new().with_prompt(prompt).interact().unwrap()
}

/// Send a notification with a dedicated message.
pub fn notify(title: &str, message: &str) -> Result<()> {
    send_notification(
        title,
        None,
        message,
        Some(
            Notification::new()
                .asynchronous(true)
                .close_button("I'm good!")
                .sound("Blow"),
        ),
    )
    .unwrap();
    Ok(())
}

/// Restart Finder, Dock, SystemUIServer so defaults take effect.
pub async fn restart_services() -> Result<()> {
    if should_not_restart_services() {
        return Ok(());
    }

    let dry_run = should_dry_run();

    // services to restart
    const SERVICES: &[&str] = &["SystemUIServer", "Dock", "Finder"];

    for svc in SERVICES {
        if dry_run {
            print_log(LogLevel::Dry, &format!("Would restart {svc}"));
        } else {
            let out = Command::new("killall").arg(svc).output().await?;
            if !out.status.success() {
                print_log(LogLevel::Error, &format!("Failed to restart {svc}"));
            } else {
                print_log(LogLevel::Info, &format!("{svc} restarted"));
            }
        }
    }

    Ok(())
}
