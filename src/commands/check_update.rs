// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::Ordering;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use clap::Args;
use reqwest;
use semver::Version;

use crate::{
    cli::atomic::should_be_quiet,
    commands::Runnable,
    log_cute, log_info,
    util::logging::{BOLD, RESET},
};

#[derive(Args, Debug)]
pub struct CheckUpdateCmd;

#[async_trait]
impl Runnable for CheckUpdateCmd {
    async fn run(&self) -> Result<()> {
        let current_version = env!("CARGO_PKG_VERSION");

        log_info!("Current version: {current_version}",);

        // fetch latest release tag from GitHub API
        let url = "https://api.github.com/repos/cutlerCLI/cutler/releases/latest";
        let client = reqwest::Client::builder()
            .user_agent("cutler-update-check")
            .build()
            .expect("Failed to build request client");
        let resp = client
            .get(url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .with_context(|| format!("Failed to fetch latest GitHub release: {url}"))?;
        let body = resp.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

        // try "tag_name" first, fallback to "name"
        let latest_version = json
            .get("tag_name")
            .and_then(|v| v.as_str())
            .or_else(|| json.get("name").and_then(|v| v.as_str()))
            .map(|s| s.trim_start_matches('v').to_string())
            .ok_or_else(|| anyhow!("Could not find latest version tag in GitHub API response"))?;

        log_info!("Latest version: {latest_version}");

        // let the comparison begin!
        let current = Version::parse(current_version).context("Could not parse current version")?;
        let latest = Version::parse(&latest_version).context("Could not parse latest version")?;

        match current.cmp(&latest) {
            Ordering::Less => {
                if !should_be_quiet() {
                    println!(
                        r#"
{BOLD}Update available:{RESET} {current_version} → {latest_version}

To update, run one of the following:

  brew update && brew upgrade cutler     # if installed via homebrew
  cargo install cutler --force           # if installed via cargo
  mise up cutler                         # if installed via mise
  cutler self-update                     # for manual installs

Or download the latest release from:
  https://github.com/cutlerCLI/cutler/releases"#
                    );
                } else {
                    log_cute!("Update available!")
                }
            }
            Ordering::Equal => {
                log_cute!("You are using the latest version.");
            }
            Ordering::Greater => {
                log_cute!(
                    "You are on a development version ({current_version}) ahead of latest release ({latest_version})."
                );
            }
        }

        Ok(())
    }
}
