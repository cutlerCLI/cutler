// SPDX-License-Identifier: Apache-2.0

use dialoguer::Confirm;
use tokio::process::Command;

use crate::{
    cli::atomic::{should_accept_all, should_dry_run, should_not_restart_services},
    log,
    util::logging::LogLevel,
};
use anyhow::Result;

/// Ask "Y/N?"; returns true if accept_all is set or the user types "y" or "Y"
pub fn confirm(prompt: &str) -> bool {
    if should_accept_all() {
        log!(LogLevel::Prompt, "{prompt} (auto-accepted)");
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
            log!(LogLevel::Dry, "Would restart {svc}");
        } else {
            match Command::new("killall").arg(svc).output().await {
                Ok(out) => {
                    if !out.status.success() {
                        log!(LogLevel::Error, "Failed to restart {svc}");
                        failed = true;
                    } else {
                        log!(LogLevel::Info, "{svc} restarted");
                    }
                }
                Err(_) => {
                    log!(LogLevel::Error, "Could not restart {svc}");
                    continue;
                }
            }
        }
    }

    if failed {
        log!(
            LogLevel::Warning,
            "Being quick with commands can cause your computer to run out of breath.",
        );
    }
}
