use crate::{
    brew::core::{backup, install_from_config, install_homebrew, is_brew_installed},
    cli::BrewCmd,
    config::get_config_path,
    util::{
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};
use anyhow::Result;

pub fn run(cmd: &BrewCmd, verbose: bool, dry_run: bool) -> Result<()> {
    if !is_brew_installed() {
        print_log(LogLevel::Warning, "Homebrew is not installed.");
        if confirm_action("Install Homebrew now?")? {
            install_homebrew(dry_run)?;
        } else {
            anyhow::bail!("Homebrew required for brew operations.");
        }
    }
    let cfg_path = get_config_path();
    match cmd {
        BrewCmd::Backup => backup(&cfg_path, verbose, dry_run),
        BrewCmd::Install => install_from_config(cfg_path, verbose, dry_run),
    }
}
