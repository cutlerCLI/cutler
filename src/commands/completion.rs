// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use clap::{Args, CommandFactory};
use clap_complete::{
    generate,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use std::io;
use tokio::task;

use crate::commands::Runnable;

/// Represents the shell types to generate completions for.
#[derive(Copy, Clone, PartialEq, Eq, clap::ValueEnum, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    PowerShell,
}

#[derive(Args, Debug)]
pub struct CompletionCmd {
    /// Your shell type.
    #[arg(value_enum)]
    shell: Shell,
}

#[async_trait]
impl Runnable for CompletionCmd {
    async fn run(&self) -> Result<()> {
        let shell = self.shell;
        task::spawn_blocking(move || -> Result<()> {
            let mut cmd = crate::cli::Args::command();
            let name = cmd.get_name().to_string();

            match shell {
                Shell::Bash => generate(Bash, &mut cmd, name, &mut io::stdout()),
                Shell::Zsh => generate(Zsh, &mut cmd, name, &mut io::stdout()),
                Shell::Fish => generate(Fish, &mut cmd, name, &mut io::stdout()),
                Shell::PowerShell => generate(PowerShell, &mut cmd, name, &mut io::stdout()),
                Shell::Elvish => generate(Elvish, &mut cmd, name, &mut io::stdout()),
            };
            Ok(())
        })
        .await??;
        Ok(())
    }
}
