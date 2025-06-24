use std::cmp::Ordering;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use clap::Args;
use semver::Version;

use crate::{
    commands::Runnable,
    util::{
        globals::should_be_quiet,
        logging::{BOLD, LogLevel, RESET, print_log},
    },
};

#[derive(Args, Debug)]
pub struct CheckUpdateCmd;

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
                        BOLD, RESET, current_version, latest_version
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
