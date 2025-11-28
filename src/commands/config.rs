// SPDX-License-Identifier: MIT OR Apache-2.0

use std::env;
use std::process::Command;

use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    cli::atomic::{should_be_quiet, should_dry_run},
    commands::Runnable,
    config::Config,
    log_cute, log_dry, log_info,
};

#[derive(Debug, Args)]
pub struct ConfigCmd {}

#[async_trait]
impl Runnable for ConfigCmd {
    fn needs_sudo(&self) -> bool {
        false
    }

    async fn run(&self, config: &Config) -> Result<()> {
        // handle dry‑run
        if should_dry_run() {
            log_dry!("Would display config from {:?}", config.path);
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
                    bail!("Failed to parse EDITOR: {e}");
                }
            };

            log_info!("Executing: {} {:?}", editor_cmd, &config.path);
            log_cute!("Opening configuration in editor. Close editor to quit.",);
            let mut command = Command::new(program);
            command.args(&args).arg(&config.path);

            let status = command.status();
            match status {
                Ok(s) if s.success() => {
                    log_info!("Opened configuration file in editor.");
                }
                Ok(s) => {
                    bail!("Editor exited with status: {s}");
                }
                Err(e) => {
                    bail!("Failed to launch editor: {e}");
                }
            }
        } else {
            if !should_be_quiet() {
                log_info!("Editor could not be found, opening normally:\n",);
            }

            // read and print the file
            let content = fs::read_to_string(&config.path).await?;

            println!("{content}");
        }

        Ok(())
    }
}
