// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{env, process::exit};

use anyhow::{Result, bail};
use nix::unistd::Uid;
use tokio::process::Command;

/// Only run the command if cutler is running as root.
/// If not running as root, rerun the command with sudo.
pub async fn run_with_root() -> Result<()> {
    if !Uid::effective().is_root() {
        let args: Vec<String> = env::args().collect();
        let status = Command::new("sudo").args(&args).status().await?;

        exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Only run the command if cutler is running as non-root.
pub fn run_with_noroot() -> Result<()> {
    if Uid::effective().is_root() {
        bail!("Do not use sudo on this command!");
    }

    Ok(())
}
