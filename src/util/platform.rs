use crate::util::logging::{LogLevel, print_log};

/// Check if we're running on macOS/Darwin
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// Ensure we're running on a compatible platform (macOS)
pub fn check_platform_compatibility() -> Result<(), anyhow::Error> {
    if !is_macos() {
        print_log(
            LogLevel::Error, 
            "cutler is designed specifically for macOS and requires macOS-specific system APIs."
        );
        print_log(
            LogLevel::Error,
            "This tool cannot function properly on other operating systems."
        );
        anyhow::bail!("Unsupported platform. cutler requires macOS to function.")
    }
    Ok(())
}

/// Get a user-friendly platform name
pub fn get_platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "Unknown"
    }
}