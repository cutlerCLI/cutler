// SPDX-License-Identifier: MIT

use std::env;
use std::process::Command;

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    cli::atomic::{should_be_quiet, should_dry_run},
    commands::Runnable,
    config::path::get_config_path,
    util::logging::{LogLevel, print_log},
};

#[derive(Debug, Default, Args)]
pub struct ConfigShowCmd {
    /// Show your configuration in $EDITOR.
    #[arg(short, long)]
    pub editor: bool,
}

#[async_trait]
impl Runnable for ConfigShowCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path().await;

        if !fs::try_exists(&config_path).await.unwrap() {
            bail!("Configuration file does not exist at {:?}", config_path);
        }

        // handle dry‑run
        if should_dry_run() {
            print_log(
                LogLevel::Dry,
                &format!("Would display config at {config_path:?}"),
            );
            return Ok(());
        }

        // show inside editor if available
        let editor = env::var("EDITOR");

        if editor.is_ok() {
            let editor_cmd = editor.unwrap();
            let status = Command::new(editor_cmd).arg(&config_path).status();
            match status {
                Ok(s) if s.success() => {
                    print_log(LogLevel::Info, "Opened configuration file in editor.");
                    return Ok(());
                }
                Ok(s) => {
                    bail!("Editor exited with status: {}", s);
                }
                Err(e) => {
                    bail!("Failed to launch editor: {}", e);
                }
            }
        }

        // read and print the file
        let content = fs::read_to_string(&config_path).await?;
        if !should_be_quiet() {
            println!("{content}");
        }

        print_log(LogLevel::Info, "Displayed configuration file.");

        Ok(())
    }
}
