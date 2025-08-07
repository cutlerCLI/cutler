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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = get_platform_name();
        assert!(!platform.is_empty());
        // This will vary by platform but should return a valid string
        assert!(matches!(platform, "macOS" | "Linux" | "Windows" | "Unknown"));
    }

    #[test]
    fn test_platform_compatibility_check() {
        // This test will pass or fail depending on the target platform
        let result = check_platform_compatibility();
        
        if cfg!(target_os = "macos") {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
}