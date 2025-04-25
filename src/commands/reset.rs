use anyhow::{Result, bail};
use std::fs;

use crate::{
    config::loader::{get_config_path, load_config},
    defaults::defaults_delete,
    domains::{collect, effective, read_current},
    snapshot::state::get_snapshot_path,
    util::{
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};

pub fn run(verbose: bool, dry_run: bool, force: bool) -> Result<()> {
    let config_path = get_config_path();
    if !config_path.exists() {
        bail!("No config file found. Please run `cutler init` first, or create a config file.");
    }

    print_log(
        LogLevel::Warning,
        "This will DELETE all settings defined in your config file.",
    );
    print_log(
        LogLevel::Warning,
        "Settings will be reset to macOS defaults, not to their previous values.",
    );

    if !force && !confirm_action("Are you sure you want to continue?")? {
        return Ok(());
    }

    let toml = load_config(&config_path)?;
    let domains = collect(&toml)?;

    for (domain, table) in domains {
        for (key, _) in table {
            let (eff_dom, eff_key) = effective(&domain, &key);

            // Only delete if currently set
            if read_current(&eff_dom, &eff_key).is_some() {
                match defaults_delete(&eff_dom, &eff_key, "Resetting", verbose, dry_run) {
                    Ok(_) => {
                        if verbose {
                            print_log(
                                LogLevel::Success,
                                &format!("Reset {}.{} to system default", eff_dom, eff_key),
                            );
                        }
                    }
                    Err(e) => {
                        print_log(
                            LogLevel::Error,
                            &format!("Failed to reset {}.{}: {}", eff_dom, eff_key, e),
                        );
                    }
                }
            } else if verbose {
                print_log(
                    LogLevel::Info,
                    &format!("Skipping {}.{} (not set)", eff_dom, eff_key),
                );
            }
        }
    }

    // remove snapshot if present
    let snap_path = get_snapshot_path();
    if snap_path.exists() {
        if dry_run {
            print_log(
                LogLevel::Info,
                &format!("Dry-run: Would remove snapshot at {:?}", snap_path),
            );
        } else if let Err(e) = fs::remove_file(&snap_path) {
            print_log(
                LogLevel::Warning,
                &format!("Failed to remove snapshot: {}", e),
            );
        } else if verbose {
            print_log(
                LogLevel::Success,
                &format!("Removed snapshot at {:?}", snap_path),
            );
        }
    }

    println!("\n🍎 Reset complete. All configured settings have been removed.");
    Ok(())
}
