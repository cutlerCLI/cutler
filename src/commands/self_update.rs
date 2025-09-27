// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use self_update::{backends::github::Update, cargo_crate_version};
use std::env;
use tokio::fs;

use crate::commands::Runnable;
use crate::util::logging::{LogLevel, print_log};

#[derive(Args, Debug)]
pub struct SelfUpdateCmd;

#[async_trait]
impl Runnable for SelfUpdateCmd {
    async fn run(&self) -> Result<()> {
        // get the path to the current executable
        let exe_path = env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy();

        // check for homebrew install
        let is_homebrew = exe_path_str == "/opt/homebrew/bin/cutler";

        // check for cargo install (e.g., ~/.cargo/bin/cutler or $CARGO_HOME/bin/cutler)
        let cargo_bin_path = if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
            format!("{cargo_home}/bin/cutler")
        } else if let Ok(home) = std::env::var("HOME") {
            format!("{home}/.cargo/bin/cutler")
        } else {
            String::new()
        };
        let is_cargo = exe_path_str == cargo_bin_path;

        if is_homebrew || is_cargo {
            print_log(
                LogLevel::Warning,
                "cutler was installed using a package manager, so cannot install updates manually.",
            );
            return Ok(());
        }

        // determine architecture for update target
        let arch = std::env::consts::ARCH;
        let target = match arch {
            "x86_64" | "x86" => "darwin-x86_64",
            "aarch64" => "darwin-arm64",
            _ => {
                bail!("Unsupported architecture for self-update: {arch}")
            }
        };

        // run the self_update updater in a blocking thread to avoid dropping a runtime in async context
        let status = tokio::task::spawn_blocking(move || {
            Update::configure()
                .repo_owner("cutlerCLI")
                .repo_name("cutler")
                .target(target)
                .bin_name("cutler")
                .bin_path_in_archive("bin/cutler")
                .show_download_progress(true)
                .current_version(cargo_crate_version!())
                .build()?
                .update()
        })
        .await??;

        if status.updated() {
            print_log(LogLevel::Info, "Binary updated, updating manpage...");

            let manpage_url = "https://raw.githubusercontent.com/cutlerCLI/cutler/refs/heads/master/man/man1/cutler.1".to_string();
            let client = reqwest::Client::builder()
                .user_agent("cutler-self-update")
                .build()?;
            let resp = client
                .get(&manpage_url)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch manpage: {}", e))?;
            let manpage_content = resp.text().await?;

            fs::create_dir_all("/usr/local/share/man/man1").await?;
            fs::write("/usr/local/share/man/man1/cutler.1", manpage_content).await?;

            print_log(
                LogLevel::Fruitful,
                &format!("cutler updated to: {}", status.version()),
            );
        } else {
            print_log(LogLevel::Fruitful, "cutler is already up to date.");
        }

        Ok(())
    }
}
