use crate::cli::{Args, Shell};
use clap::CommandFactory;
use clap_complete::{
    generate,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use std::io;

pub fn generate_completion(shell: Shell) -> anyhow::Result<()> {
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
}
