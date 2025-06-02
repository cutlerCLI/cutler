use anyhow::{Context, Result, anyhow};
use self_update::backends::github::Update;
use self_update::cargo_crate_version;
use semver::Version;
use std::cmp::Ordering;
use toml::Value;
use ureq;

use crate::util::logging::{LogLevel, print_log};

pub async fn run_check_update(verbose: bool) -> Result<()> {
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
    let body = tokio::task::spawn_blocking(move || {
        let response = ureq::get(url)
            .call()
            .map_err(|e| anyhow!("Failed to fetch remote Cargo.toml: {}", e))?;
        response
            .into_body()
            .read_to_string()
            .context("Failed to read response body")
    })
    .await??;

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
            println!("\nTo update, run one of the following:\n");
            println!("  brew update && brew upgrade cutler     # if installed via Homebrew");
            println!("  cargo install cutler --force           # if installed via Cargo");
            println!("  cutler self-update                     # if installed manually");
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

pub fn run_self_update() -> anyhow::Result<()> {
    use std::env;

    // get the path to the current executable
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    // check for homebrew install
    let is_homebrew = exe_path_str == "/opt/homebrew/bin/cutler";

    // check for cargo install (e.g., ~/.cargo/bin/cutler or $CARGO_HOME/bin/cutler)
    let cargo_bin_path = if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        format!("{}/bin/cutler", cargo_home)
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{}/.cargo/bin/cutler", home)
    } else {
        String::new()
    };
    let is_cargo = exe_path_str == cargo_bin_path;

    if is_homebrew || is_cargo {
        println!(
            "cutler was installed using a package manager, so cannot install updates manually."
        );
        return Ok(());
    }

    // Determine architecture for update target
    let arch = std::env::consts::ARCH;
    let target = match arch {
        "x86_64" | "x86" => "darwin-x86_64",
        "aarch64" => "darwin-arm64",
        _ => {
            println!("Unsupported architecture for self-update: {}", arch);
            return Ok(());
        }
    };

    let status = Update::configure()
        .repo_owner("hitblast")
        .repo_name("cutler")
        .target(target)
        .bin_name("cutler")
        .bin_path_in_archive("bin/cutler")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    if status.updated() {
        println!("✅ cutler updated to: {}", status.version());
    } else {
        println!("cutler is already up to date.");
    }

    Ok(())
}
