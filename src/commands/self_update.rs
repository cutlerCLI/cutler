use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use self_update::backends::github::Update;
use self_update::cargo_crate_version;

use crate::commands::Runnable;
use crate::util::logging::{LogLevel, print_log};

#[derive(Args, Debug)]
pub struct SelfUpdateCmd;

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
                print_log(
                    LogLevel::Error,
                    &format!("Unsupported architecture for self-update: {}", arch),
                );
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
