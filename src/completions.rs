use clap::CommandFactory;
use clap_complete::{
    generate_to,
    shells::{Bash, Zsh},
};
use std::io;
use std::path::Path;

use crate::cli::{Cli, Shell};

/// Generates completion script for the specified shell
pub fn generate_completion(shell: Shell, output_dir: &Path) -> io::Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    match shell {
        Shell::Bash => {
            let file = format!("{}.bash", name);
            generate_to(Bash, &mut cmd, name.clone(), output_dir)?;
            print_completion_info("Bash", &file, output_dir);
        }
        Shell::Zsh => {
            let file = format!("_{}", name);
            generate_to(Zsh, &mut cmd, name.clone(), output_dir)?;
            print_completion_info("Zsh", &file, output_dir);
        }
    }

    Ok(())
}

fn print_completion_info(shell_name: &str, file_name: &str, output_dir: &Path) {
    println!(
        "{} completion script written to: {}/{}",
        shell_name,
        output_dir.display(),
        file_name
    );

    if shell_name == "Bash" {
        println!("\nTo use it temporarily, run:");
        println!("  source {}/{}", output_dir.display(), file_name);
        println!("\nFor permanent use, add to your ~/.bashrc:");
        println!("  source {}/{}", output_dir.display(), file_name);
    } else if shell_name == "Zsh" {
        println!("\nTo use it, copy the file to a directory in your $fpath:");
        println!("  cp {}/{} ~/.zfunc/", output_dir.display(), file_name);
        println!("\nMake sure ~/.zfunc is in your fpath in ~/.zshrc:");
        println!("  fpath=(~/.zfunc $fpath)");
        println!("  autoload -U compinit && compinit");
    }
}
