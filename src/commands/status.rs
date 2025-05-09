use crate::{
    config::loader::{get_config_path, load_config},
    defaults::normalize,
    domains::{collect, effective, read_current},
    util::logging::{BOLD, GREEN, LogLevel, RED, RESET, print_log},
};
use anyhow::{Result, bail};

pub async fn run(prompt_mode: bool, verbose: bool) -> Result<()> {
    let config_path = get_config_path();
    if !config_path.exists() {
        if !prompt_mode {
            bail!("No config file found. Please run `cutler init` first, or create a config file.");
        } else {
            return Ok(());
        }
    }

    let toml = load_config(&config_path).await?;
    let domains = collect(&toml)?;

    // flatten all settings into a list for parallel processing
    let entries: Vec<(String, String, toml::Value)> = domains
        .into_iter()
        .flat_map(|(domain, table)| {
            table
                .into_iter()
                .map(move |(key, value)| (domain.clone(), key.clone(), value.clone()))
        })
        .collect();

    // prompt mode: bail out on first mismatch, otherwise stay silent
    if prompt_mode {
        let diverges = entries.iter().any(|(domain, key, value)| {
            let (eff_dom, eff_key) = effective(domain, key);
            let desired = normalize(value);
            let current = read_current(&eff_dom, &eff_key).unwrap_or_else(|| "Not set".into());
            current != desired
        });
        if diverges {
            print_log(
                LogLevel::Warning,
                "cutler: Your system has diverged from config; run `cutler apply`",
            );
        }
        return Ok(());
    }

    // normal mode: collect results sequentially
    let mut outcomes = Vec::with_capacity(entries.len());
    for (domain, key, value) in entries.iter() {
        let (eff_dom, eff_key) = effective(domain, key);
        let desired = normalize(value);
        let current = read_current(&eff_dom, &eff_key).unwrap_or_else(|| "Not set".into());
        let is_diff = current != desired;
        outcomes.push((eff_dom, eff_key, desired, current, is_diff));
    }

    let mut any_diff = false;
    for (eff_dom, eff_key, desired, current, is_diff) in outcomes {
        if is_diff {
            any_diff = true;
            println!(
                "{}{}.{}: should be {} (currently {}{}{}){}",
                BOLD, eff_dom, eff_key, desired, RED, current, RESET, RESET,
            );
        } else if verbose {
            println!(
                "{}{}.{}: {} (matches desired){}",
                GREEN, eff_dom, eff_key, current, RESET
            );
        }
    }

    if !any_diff {
        println!("\n🍎 All settings already match your configuration.");
    } else {
        println!("\nRun `cutler apply` to apply these changes from your config.");
    }

    Ok(())
}
