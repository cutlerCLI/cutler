use std::env;
use std::fs;
use std::io::{self};
use std::path::PathBuf;

use anyhow::Context;
use toml::Value;

use crate::logging::{print_log, LogLevel};

/// Returns the path to the configuration file, looking at XDG_CONFIG_HOME or HOME.
pub fn get_config_path() -> PathBuf {
    if let Some(xdg_config) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join("cutler").join("config.toml")
    } else if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("cutler")
            .join("config.toml")
    } else {
        PathBuf::from("config.toml")
    }
}

/// Returns the path where the snapshot is stored.
pub fn get_snapshot_path() -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home).join(".cutler_snapshot")
    } else {
        PathBuf::from(".cutler_snapshot")
    }
}

/// Helper: Read and parse the configuration file at a given path.
pub fn load_config(path: &PathBuf) -> Result<Value, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let parsed: Value = content.parse::<Value>()?;
    Ok(parsed)
}

/// If no config file is present, create an example config.
pub fn create_example_config(path: &PathBuf, verbose: bool) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let example = r#"
# This is just an example for you to get started with.
# Learn more: https://github.com/hitblast/cutler

[dock]
tilesize = 50
autohide = true

[finder]
AppleShowAllFiles = true
CreateDesktop = false

[NSGlobalDomain]
ApplePressAndHoldEnabled = true

# Also valid: If you want to store a prefixed key under NSGlobalDomain,
# you can provide a subdomain. In the example below, the key will become
# "com.apple.mouse.linear".
[NSGlobalDomain.com.apple.mouse]
linear = true
    "#;
    if let Ok(_) = fs::write(path, example.trim_start())
        .with_context(|| format!("Failed to write example config file at {:?}", path))
    {
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Example config created at: {:?}", path),
            );
        } else {
            println!("🍎 Example config written to {:?}", path);
        }
    }

    Ok(())
}
