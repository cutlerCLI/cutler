use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use clap::Args;
use self_update::backends::github::Update;
use self_update::cargo_crate_version;
use semver::Version;
use std::cmp::Ordering;
use ureq;

use crate::commands::Runnable;
use crate::util::globals::should_be_quiet;
use crate::util::logging::{LogLevel, print_log};

#[derive(Args, Debug)]
pub struct CheckUpdateCmd;

#[derive(Args, Debug)]
pub struct SelfUpdateCmd;

#[async_trait]
impl Runnable for CheckUpdateCmd {
    async fn run(&self) -> Result<()> {
        let current_version = env!("CARGO_PKG_VERSION");

        print_log(
            LogLevel::Info,
            &format!("Current version: {}", current_version),
        );

        // fetch latest release tag from GitHub API
        let url = "https://api.github.com/repos/hitblast/cutler/releases/latest";
        let latest_version: String = tokio::task::spawn_blocking(move || {
            let response = ureq::get(url)
                .header("Accept", "application/vnd.github.v3+json")
                .header("User-Agent", "cutler-update-check")
                .call()
                .map_err(|e| anyhow!("Failed to fetch latest GitHub release: {}", e))?;

            let body_reader = response
                .into_body()
                .read_to_string()
                .map_err(|e| anyhow!("Failed to read GitHub API response body: {}", e))?;

            let json: serde_json::Value = serde_json::from_str(&body_reader)
                .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

            // try "tag_name" first, fallback to "name"
            json.get("tag_name")
                .and_then(|v| v.as_str())
                .or_else(|| json.get("name").and_then(|v| v.as_str()))
                .map(|s| s.trim_start_matches('v').to_string())
                .ok_or_else(|| anyhow!("Could not find latest version tag in GitHub API response"))
        })
        .await??;

        print_log(
            LogLevel::Info,
            &format!("Latest version: {}", latest_version),
        );

        // let the comparison begin!
        let current = Version::parse(current_version).context("Could not parse current version")?;
        let latest = Version::parse(&latest_version).context("Could not parse latest version")?;

        match current.cmp(&latest) {
            Ordering::Less => {
                if !should_be_quiet() {
                    println!(
                        r#"
{}Update available:{} {} → {}

To update, run one of the following:

  brew update && brew upgrade cutler     # if installed via homebrew
  cargo install cutler --force           # if installed via cargo
  mise up cutler                         # if installed via mise
  cutler self-update                     # for manual installs

Or download the latest release from:
  https://github.com/hitblast/cutler/releases"#,
                        crate::util::logging::BOLD,
                        crate::util::logging::RESET,
                        current_version,
                        latest_version
                    );
                } else {
                    println!("Update available!")
                }
            }
            Ordering::Equal => {
                print_log(LogLevel::Fruitful, "You are using the latest version.");
            }
            Ordering::Greater => {
                print_log(
                    LogLevel::Fruitful,
                    &format!(
                        "You are on a development version ({}) ahead of latest release ({}).",
                        current_version, latest_version
                    ),
                );
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Runnable for SelfUpdateCmd {
    async fn run(&self) -> Result<()> {
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

        // determine architecture for update target
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

        if !should_be_quiet() {
            if status.updated() {
                println!("✅ cutler updated to: {}", status.version());
            } else {
                println!("cutler is already up to date.");
            }
        }

        Ok(())
    }
}
