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
pub struct SelfUpdateCmd {
    /// Do not install/update manpage during the update procedure.
    #[arg(long)]
    no_man: bool,
}

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
        } else if let Some(home_dir) = dirs::home_dir() {
            format!("{}/.cargo/bin/cutler", home_dir.to_string_lossy())
        } else {
            String::new()
        };
        let is_cargo = exe_path_str == cargo_bin_path;

        // check for mise install
        let is_mise = exe_path_str.contains(".local/share/mise/installs/cargo-cutler");

        if is_homebrew || is_cargo || is_mise {
            print_log(
                LogLevel::Warning,
                "cutler was installed using a package manager, so cannot install updates manually.",
            );
            return Ok(());
        }

        // finally, check if cutler is where it is supposed to be
        if exe_path_str != "/usr/local/bin/cutler" {
            print_log(
                LogLevel::Warning,
                "cutler is currently installed in a custom path. Please note that the manpage will still be installed in: /usr/local/share/man/man1/cutler.1",
            );
            print_log(
                LogLevel::Warning,
                "If you wish to skip this behavior, use: cutler self-update --no-man",
            );
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
            if !self.no_man {
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
            }
        } else {
            print_log(LogLevel::Fruitful, "cutler is already up to date.");
            return Ok(());
        }

        print_log(
            LogLevel::Fruitful,
            &format!("cutler updated to: {}", status.version()),
        );

        Ok(())
    }
}
