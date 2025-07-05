use anyhow::bail;
use nix::unistd::Uid;

/// Only run the command if cutler is running as root.
pub fn run_with_root() -> Result<(), anyhow::Error> {
    if !Uid::effective().is_root() {
        bail!("You must run this command with sudo.");
    }

    Ok(())
}

/// Only run the command if cutler is running as non-root.
pub fn run_with_noroot() -> Result<(), anyhow::Error> {
    if Uid::effective().is_root() {
        bail!("Do not use sudo on this command!");
    }

    Ok(())
}
