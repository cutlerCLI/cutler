use anyhow::{Result, anyhow};
use tokio::fs;

use crate::{
    config::loader::get_config_path,
    util::{
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};

pub async fn run(basic: bool, verbose: bool, dry_run: bool, force: bool) -> Result<()> {
    let config_path = get_config_path();

    let exists = fs::metadata(&config_path).await.is_ok();
    if exists && !force {
        print_log(
            LogLevel::Warning,
            &format!("Configuration file already exists at {:?}", config_path),
        );
        if !confirm_action("Do you want to overwrite it?")? {
            return Err(anyhow!("Configuration initialization aborted."));
        }
    }

    // ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        if dry_run {
            print_log(
                LogLevel::Dry,
                &format!("Would create directory: {:?}", parent),
            );
        } else {
            if verbose {
                print_log(
                    LogLevel::Info,
                    &format!("Creating parent dir: {:?}", parent),
                );
            }
            fs::create_dir_all(parent).await?;
        }
    }

    // default TOML template
    let default_cfg = match basic {
        true => {
            if verbose {
                print_log(LogLevel::Info, "Choosing basic configuration...")
            }
            include_str!("../../examples/basic.toml")
        }
        _ => {
            if verbose {
                print_log(
                    LogLevel::Info,
                    "No `--basic` flag, defaulting to advanced configuration...",
                )
            }
            include_str!("../../examples/advanced.toml")
        }
    };

    if dry_run {
        print_log(
            LogLevel::Dry,
            &format!("Would write configuration to {:?}", config_path),
        );
        print_log(
            LogLevel::Dry,
            &format!("Configuration content:\n{}", default_cfg),
        );
    } else {
        fs::write(&config_path, default_cfg)
            .await
            .map_err(|e| anyhow!("Failed to write configuration to {:?}: {}", config_path, e))?;

        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Configuration file created at: {:?}", config_path),
            );
        } else {
            println!(
                "🍎 New configuration created at {:?}\nReview and customize this file before running cutler again.",
                config_path
            );
        }
    }

    Ok(())
}
