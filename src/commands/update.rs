use anyhow::{Context, Result, anyhow};
use semver::Version;
use std::cmp::Ordering;
use toml::Value;
use ureq;

use crate::util::logging::{LogLevel, print_log};

/// Checks for updates to cutler by fetching the latest version from GitHub's Cargo.toml
pub fn run(verbose: bool) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    if verbose {
        print_log(
            LogLevel::Info,
            &format!("Current version: {}", current_version),
        );
        print_log(LogLevel::Info, "Checking for updates...");
    } else {
        println!("Checking for updates...");
    }

    // fetch remote Cargo.toml
    // URL: https://github.com/hitblast/cutler
    let url = "https://raw.githubusercontent.com/hitblast/cutler/main/Cargo.toml";
    let response = ureq::get(url)
        .call()
        .map_err(|e| anyhow!("Failed to fetch remote Cargo.toml: {}", e))?;

    let body = response
        .into_body()
        .read_to_string()
        .context("Failed to read response body")?;

    // parse it as TOML and extract `package.version`
    let toml_val: Value = body.parse().context("Failed to parse remote Cargo.toml")?;

    let latest_version = toml_val
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid version in remote Cargo.toml"))?;

    if verbose {
        print_log(
            LogLevel::Info,
            &format!("Latest version: {}", latest_version),
        );
    }

    // let the comparison begin!
    let current = Version::parse(current_version).context("Could not parse current version")?;
    let latest = Version::parse(latest_version).context("Could not parse latest version")?;

    match current.cmp(&latest) {
        Ordering::Less => {
            // update available
            println!(
                "\n{}Update available:{} {} → {}",
                crate::util::logging::BOLD,
                crate::util::logging::RESET,
                current_version,
                latest_version
            );
            println!("\nTo update, run one of the following:");
            println!("  brew upgrade hitblast/tap/cutler    # if installed via Homebrew");
            println!("  cargo install cutler --force        # if installed via Cargo");
            println!("\nOr download the latest release from:");
            println!("  https://github.com/hitblast/cutler/releases");
        }
        Ordering::Equal => {
            print_log(LogLevel::Success, "You are using the latest version.");
        }
        Ordering::Greater => {
            print_log(
                LogLevel::Info,
                &format!(
                    "You are on a development version ({}) ahead of latest release ({}).",
                    current_version, latest_version
                ),
            );
        }
    }

    Ok(())
}
