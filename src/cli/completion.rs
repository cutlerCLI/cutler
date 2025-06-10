use crate::cli::Args;
use clap::{CommandFactory, ValueEnum};
use clap_complete::{
    generate,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use std::io;
use tokio::task;

/// Generates a shell completion script given the shell type.
pub async fn generate_completion(shell: Shell) -> anyhow::Result<()> {
    task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut cmd = Args::command();
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

/// Represents the shell types to generate completions for.
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    PowerShell,
}
