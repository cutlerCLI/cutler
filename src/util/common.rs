use anyhow::bail;
use nix::unistd::Uid;

/// Checks if the current user is root.
pub fn is_root() -> Result<(), anyhow::Error> {
    if !Uid::effective().is_root() {
        bail!("You must run this command with sudo.");
    }

    Ok(())
}
