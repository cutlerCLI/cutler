use anyhow::bail;
#[cfg(feature = "macos-deps")]
use nix::unistd::Uid;

/// Only run the command if cutler is running as root.
#[cfg(feature = "macos-deps")]
pub fn run_with_root() -> Result<(), anyhow::Error> {
    if !Uid::effective().is_root() {
        bail!("You must run this command with sudo.");
    }

    Ok(())
}

/// Only run the command if cutler is running as non-root.
#[cfg(feature = "macos-deps")]
pub fn run_with_noroot() -> Result<(), anyhow::Error> {
    if Uid::effective().is_root() {
        bail!("Do not use sudo on this command!");
    }

    Ok(())
}

/// Fallback implementation for non-macOS platforms.
#[cfg(not(feature = "macos-deps"))]
pub fn run_with_root() -> Result<(), anyhow::Error> {
    bail!("Root privilege checking is not supported on this platform.")
}

/// Fallback implementation for non-macOS platforms.
#[cfg(not(feature = "macos-deps"))]
pub fn run_with_noroot() -> Result<(), anyhow::Error> {
    // On non-macOS platforms, just allow execution
    Ok(())
}
