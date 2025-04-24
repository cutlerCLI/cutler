use crate::{
    config::loader::{get_config_path, load_config},
    defaults::normalize,
    domains::{collect, effective, read_current},
    util::logging::{BOLD, GREEN, RED, RESET},
};
use anyhow::{Result, bail};

pub fn run(verbose: bool) -> Result<()> {
    let config_path = get_config_path();
    if !config_path.exists() {
        bail!("No config file found. Please run `cutler init` first, or create a config file.");
    }

    let toml = load_config(&config_path)?;
    let domains = collect(&toml)?;

    println!("\n{} Current Status:", BOLD);
    let mut any_diff = false;

    for (domain, table) in domains {
        for (key, value) in table {
            let (eff_dom, eff_key) = effective(&domain, &key);
            let desired = normalize(&value);
            let current = read_current(&eff_dom, &eff_key).unwrap_or_else(|| "Not set".into());

            if current != desired {
                any_diff = true;
                println!(
                    "{}{}.{}: should be {} (currently {}{}{}){}",
                    BOLD, eff_dom, eff_key, desired, RED, current, RESET, RESET
                );
            } else if verbose {
                println!(
                    "{}{}.{}: {} (matches desired){}",
                    GREEN, eff_dom, eff_key, current, RESET
                );
            }
        }
    }

    if !any_diff {
        println!("🍎 All settings already match your configuration.");
    } else {
        println!("\nRun `cutler apply` to apply these changes from your config.");
    }

    Ok(())
}
