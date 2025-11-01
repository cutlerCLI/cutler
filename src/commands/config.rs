// SPDX-License-Identifier: GPL-3.0-or-later

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

#[derive(Debug, Args)]
pub struct ConfigCmd {}

#[async_trait]
impl Runnable for ConfigCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path().await?;

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

        if let Ok(editor_cmd) = editor {
            // Split the editor command into program and args, respecting quoted arguments
            let parsed = shell_words::split(&editor_cmd);
            let (program, args) = match parsed {
                Ok(mut parts) if !parts.is_empty() => {
                    let prog = parts.remove(0);
                    (prog, parts)
                }
                Ok(_) => {
                    bail!("EDITOR environment variable is empty.");
                }
                Err(e) => {
                    bail!("Failed to parse EDITOR: {}", e);
                }
            };

            print_log(
                LogLevel::Info,
                &format!("Executing: {} {:?}", editor_cmd, config_path),
            );
            print_log(
                LogLevel::Fruitful,
                "Opening configuration in editor. Close editor to quit.",
            );
            let mut command = Command::new(program);
            command.args(&args).arg(&config_path);

            let status = command.status();
            match status {
                Ok(s) if s.success() => {
                    print_log(LogLevel::Info, "Opened configuration file in editor.");
                }
                Ok(s) => {
                    bail!("Editor exited with status: {}", s);
                }
                Err(e) => {
                    bail!("Failed to launch editor: {}", e);
                }
            }
        } else {
            print_log(
                LogLevel::Info,
                "Editor could not be found, opening normally:\n",
            );
            // read and print the file
            let content = fs::read_to_string(&config_path).await?;
            if !should_be_quiet() {
                println!("{content}");
            }
        }

        Ok(())
    }
}
