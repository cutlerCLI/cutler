// SPDX-License-Identifier: MIT OR Apache-2.0

use std::time::Duration;

use anyhow::{Result, bail};
use tokio::process::Command;

use crate::{cli::atomic::should_dry_run, log_cute, log_dry, log_warn, util::io::confirm};

/// Checks if Xcode CLT is installed on the device.
async fn check_installed() -> bool {
    let output = Command::new("xcode-select").arg("-p").output().await;
    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            !path.is_empty()
        }
        _ => false,
    }
}

/// Helper for: `ensure_brew()`
/// Ensures Xcode Command Line Tools are installed.
/// If not, prompts the user to install them (unless `dry_run`).
pub async fn ensure_xcode_clt() -> Result<()> {
    if check_installed().await {
        return Ok(());
    }

    if should_dry_run() {
        log_dry!("Would install Xcode Command Line Tools (not detected)");
        return Ok(());
    }

    log_warn!("Xcode CLT is not installed.");

    if confirm("Install Xcode Command Line Tools now?") {
        let status = Command::new("xcode-select")
            .arg("--install")
            .status()
            .await?;

        if !status.success() {
            bail!(
                "Failed to launch Xcode Command Line Tools installer. Try manually installing it using `xcode-select --install`."
            );
        }

        log_warn!("Waiting for installation to complete...");

        // wait for 60 minutes for the user to finish installation
        // otherwise, bail out
        for _ in 0..720 {
            tokio::time::sleep(Duration::from_millis(5000)).await;

            // loop checks here
            if check_installed().await {
                log_cute!("Xcode Command Line tools installed.");
                return Ok(());
            }
        }

        bail!(
            "Timed out. Re-run this command once installation completes.\nIf there was an error during installation, try running `xcode-select --install` again."
        );
    }
    bail!(
        "Xcode Command Line Tools are required for Homebrew operations, but were not found. Aborting."
    );
}
