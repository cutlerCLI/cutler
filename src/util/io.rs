// SPDX-License-Identifier: Apache-2.0

use dialoguer::Confirm;
use tokio::process::Command;

use crate::{
    cli::atomic::{should_accept_all, should_dry_run, should_not_restart_services},
    util::logging::{LogLevel, print_log},
};
use anyhow::Result;

/// Ask "Y/N?"; returns true if accept_all is set or the user types "y" or "Y"
pub fn confirm(prompt: &str) -> bool {
    if should_accept_all() {
        print_log(LogLevel::Prompt, &format!("{prompt} (auto-accepted)"));
        return true;
    }

    Confirm::new()
        .with_prompt(prompt)
        .interact()
        .unwrap_or_default()
}

/// Run the `open` shell command on a given argument.
pub async fn open(arg: &str) -> Result<()> {
    let _ = Command::new("open")
        .arg(arg)
        .status()
        .await
        .expect("Failed to run `open`.");

    Ok(())
}

/// Restart Finder, Dock, SystemUIServer so defaults take effect.
pub async fn restart_services() {
    if should_not_restart_services() {
        return;
    }

    let dry_run = should_dry_run();

    // services to restart
    const SERVICES: &[&str] = &["SystemUIServer", "Dock", "Finder"];

    let mut failed: bool = false;

    for svc in SERVICES {
        if dry_run {
            print_log(LogLevel::Dry, &format!("Would restart {svc}"));
        } else {
            match Command::new("killall").arg(svc).output().await {
                Ok(out) => {
                    if !out.status.success() {
                        print_log(LogLevel::Error, &format!("Failed to restart {svc}"));
                        failed = true;
                    } else {
                        print_log(LogLevel::Info, &format!("{svc} restarted"));
                    }
                }
                Err(_) => {
                    print_log(LogLevel::Error, &format!("Could not restart {svc}"));
                    continue;
                }
            }
        }
    }

    if failed {
        print_log(
            LogLevel::Warning,
            "Being quick with commands can cause your computer to run out of breath.",
        );
    }
}
