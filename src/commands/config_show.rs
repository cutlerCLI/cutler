use anyhow::{Result, bail};
use tokio::fs;

use crate::{
    config::loader::get_config_path,
    util::logging::{LogLevel, print_log},
};

pub async fn run(verbose: bool, dry_run: bool, quiet: bool) -> Result<()> {
    let config_path = get_config_path();

    if !config_path.exists() {
        bail!("Configuration file does not exist at {:?}", config_path);
    }

    // handle dry‑run
    if dry_run {
        if !quiet {
            print_log(
                LogLevel::Dry,
                &format!("Would display config at {:?}", config_path),
            );
        }
        return Ok(());
    }

    // read and print the file
    let content = fs::read_to_string(&config_path).await?;
    if !quiet {
        println!("{}", content);
    }

    if verbose && !quiet {
        print_log(LogLevel::Info, "Displayed configuration file.");
    }

    Ok(())
}
