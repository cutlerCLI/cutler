// Imports.
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use toml::Value;

// ANSI color escape codes – exported so main.rs can use them.
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const RESET: &str = "\x1b[0m";

/// Returns the path to the config file, respecting XDG_CONFIG_HOME if available.
pub fn get_config_path() -> PathBuf {
    if let Some(xdg_config) = env::var_os("XDG_CONFIG_HOME") {
        let mut config_path = PathBuf::from(xdg_config);
        config_path.push("cutler");
        config_path.push("config.toml");
        config_path
    } else if let Some(home) = env::var_os("HOME") {
        let mut config_path = PathBuf::from(home);
        config_path.push(".config");
        config_path.push("cutler");
        config_path.push("config.toml");
        config_path
    } else {
        // Fallback to a relative path if HOME is not set.
        PathBuf::from("config.toml")
    }
}

/// When no config file is present, create an example one.
pub fn create_example_config(path: &PathBuf, verbose: bool) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let example = r#"
# This is just an example for you to get started with.
# Learn more: https://github.com/hitblast/cutler

[dock]
tilesize = 50
autohide = true

[finder]
AppleShowAllFiles = true
CreateDesktop = false

[NSGlobalDomain]
ApplePressAndHoldEnabled = true

# Also valid: If you want to store a prefixed key under NSGlobalDomain,
# you can provide a subdomain. In the example below, the key will become
# "com.apple.mouse.linear".
[NSGlobalDomain.com.apple.mouse]
linear = true
    "#;
    fs::write(path, example.trim_start())?;
    if verbose {
        println!(
            "{}[SUCCESS]{} Example config created at: {:?}",
            GREEN, RESET, path
        );
    } else {
        println!("🍺 Example config written to {:?}", path);
    }
    Ok(())
}

/// Recursively flattens a TOML table into a vector of (domain, settings_table) pairs.
/// For example, [menuextra.clock] becomes a domain string "menuextra.clock".
fn flatten_domains(
    prefix: Option<String>,
    table: &toml::value::Table,
    dest: &mut Vec<(String, toml::value::Table)>,
) {
    // Temporary table to collect non-table keys.
    let mut flat_table = toml::value::Table::new();
    let mut nested_tables = toml::value::Table::new();

    for (key, value) in table {
        match value {
            Value::Table(_) => {
                nested_tables.insert(key.clone(), value.clone());
            }
            _ => {
                flat_table.insert(key.clone(), value.clone());
            }
        }
    }

    if !flat_table.is_empty() {
        let domain = match &prefix {
            Some(p) => p.clone(),
            None => String::new(),
        };
        dest.push((domain, flat_table));
    }

    for (key, value) in nested_tables {
        if let Value::Table(inner) = value {
            let new_prefix = match &prefix {
                Some(p) if !p.is_empty() => format!("{}.{}", p, key),
                _ => key.clone(),
            };
            flatten_domains(Some(new_prefix), &inner, dest);
        }
    }
}

/// Given a flattened domain (from config) and a key, returns the effective
/// domain and key to use with defaults.
///
/// • For entries not beginning with "NSGlobalDomain", returns:
///      ("com.apple.<domain>", key)
/// • For an entry exactly equal to "NSGlobalDomain", returns:
///      ("NSGlobalDomain", key)
/// • For an entry that starts with "NSGlobalDomain.", returns:
///      ("NSGlobalDomain", "<rest-of-domain>.<key>")
fn get_effective_domain_and_key(domain: &str, key: &str) -> (String, String) {
    if domain == "NSGlobalDomain" {
        ("NSGlobalDomain".to_string(), key.to_string())
    } else if domain.starts_with("NSGlobalDomain.") {
        let remainder = domain.strip_prefix("NSGlobalDomain.").unwrap_or("");
        if remainder.is_empty() {
            ("NSGlobalDomain".to_string(), key.to_string())
        } else {
            (
                "NSGlobalDomain".to_string(),
                format!("{}.{}", remainder, key),
            )
        }
    } else {
        (format!("com.apple.{}", domain), key.to_string())
    }
}

/// Checks whether a given domain exists using `defaults read`.
pub fn check_domain_exists(full_domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Execute: defaults read <full_domain>
    let output = Command::new("defaults")
        .arg("read")
        .arg(full_domain)
        .output()?;
    if !output.status.success() {
        return Err(format!("Domain '{}' does not exist. Aborting.", full_domain).into());
    }
    Ok(())
}

/// Reads the config file, validates domains, and applies each default via `defaults write`.
pub fn apply_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        if verbose {
            println!("Config file not found at {:?}.", config_path);
            print!("Would you like to create an example config file? [y/N]: ");
        } else {
            print!("No config found. Create example? [y/N]: ");
        }
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            create_example_config(&config_path, verbose)?;
            return Ok(());
        } else {
            return Err("No config file present. Exiting.".into());
        }
    }

    let content = fs::read_to_string(&config_path)?;
    let parsed: Value = content.parse::<Value>()?;
    let root_table = parsed
        .as_table()
        .ok_or("Invalid config format: expected table at top level")?;

    let mut domains: Vec<(String, toml::value::Table)> = Vec::new();
    for (key, value) in root_table {
        if let Value::Table(inner) = value {
            flatten_domains(Some(key.clone()), inner, &mut domains);
        }
    }

    for (domain, settings_table) in domains {
        // Special NSGlobalDomain support.
        // For non-global entries we build the domain normally.
        // For NSGlobalDomain entries, the effective domain will be "NSGlobalDomain"
        // and any dotted suffix in the flattened domain is prepended to each key.
        // (See get_effective_domain_and_key.)
        // Check whether the effective domain exists.
        // (Note: defaults read NSGlobalDomain works as expected.)
        let effective_domain = if domain.starts_with("NSGlobalDomain") {
            "NSGlobalDomain".to_string()
        } else {
            format!("com.apple.{}", domain)
        };
        check_domain_exists(&effective_domain)?;

        // For each key in the settings table, determine the effective domain and key.
        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, &key);
            let (flag, value_str) = match value {
                Value::Boolean(b) => ("-bool", b.to_string()),
                Value::Integer(i) => ("-int", i.to_string()),
                Value::Float(f) => ("-float", f.to_string()),
                Value::String(s) => ("-string", s.clone()),
                _ => {
                    return Err(format!(
                        "Unsupported type for key '{}' in domain '{}'",
                        key, domain
                    )
                    .into())
                }
            };

            if verbose {
                println!(
                    "Applying: defaults write {} \"{}\" {} \"{}\"",
                    eff_domain, eff_key, flag, value_str
                );
            }

            let output = Command::new("defaults")
                .arg("write")
                .arg(&eff_domain)
                .arg(&eff_key)
                .arg(flag)
                .arg(&value_str)
                .output()?;

            if !output.status.success() {
                eprintln!(
                    "{}[ERROR]{} Failed to apply setting '{}' for {}.",
                    RED, RESET, eff_key, eff_domain
                );
            } else if verbose {
                println!(
                    "{}[SUCCESS]{} Applied setting '{}' for {}.",
                    GREEN, RESET, eff_key, eff_domain
                );
            }
        }
        if !verbose {
            println!("Updated {}", effective_domain);
        }
    }
    Ok(())
}

/// Reads the config file, validates domains, and unapplies each default via `defaults delete`.
pub fn unapply_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Err(format!("Config file not found at {:?}.", config_path).into());
    }

    let content = fs::read_to_string(&config_path)?;
    let parsed: Value = content.parse::<Value>()?;
    let root_table = parsed
        .as_table()
        .ok_or("Invalid config format: expected table at top level")?;

    let mut domains: Vec<(String, toml::value::Table)> = Vec::new();
    for (key, value) in root_table {
        if let Value::Table(inner) = value {
            flatten_domains(Some(key.clone()), inner, &mut domains);
        }
    }

    for (domain, settings_table) in domains {
        let effective_domain = if domain.starts_with("NSGlobalDomain") {
            "NSGlobalDomain".to_string()
        } else {
            format!("com.apple.{}", domain)
        };
        check_domain_exists(&effective_domain)?;

        for (key, _value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, &key);
            if verbose {
                println!("Unapplying: defaults delete {} \"{}\"", eff_domain, eff_key);
            }

            let output = Command::new("defaults")
                .arg("delete")
                .arg(&eff_domain)
                .arg(&eff_key)
                .output()?;

            if !output.status.success() {
                eprintln!(
                    "{}[ERROR]{} Failed to unapply setting '{}' for {}.",
                    RED, RESET, eff_key, eff_domain
                );
            } else if verbose {
                println!(
                    "{}[SUCCESS]{} Unapplied setting '{}' for {}.",
                    GREEN, RESET, eff_key, eff_domain
                );
            }
        }
        if !verbose {
            println!("Reverted {}", effective_domain);
        }
    }
    Ok(())
}

/// Deletes the configuration file if it exists.
pub fn delete_config(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if config_path.exists() {
        fs::remove_file(&config_path)?;
        if verbose {
            println!(
                "{}[SUCCESS]{} Configuration file deleted from: {:?}",
                GREEN, RESET, config_path
            );
        } else {
            println!("🗑️ Config deleted from {:?}", config_path);
        }
    } else {
        if verbose {
            println!(
                "{}[SUCCESS]{} No configuration file found at: {:?}",
                GREEN, RESET, config_path
            );
        } else {
            println!("🍺 No config file to delete.");
        }
    }
    Ok(())
}

/// Kills (restarts) Finder, Dock, and SystemUIServer to refresh settings.
pub fn restart_system_services(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    for service in &["Finder", "Dock", "SystemUIServer"] {
        let output = Command::new("killall").arg(service).output()?;
        if !output.status.success() {
            eprintln!(
                "{}[ERROR]{} Failed to restart {}. Try restarting manually.",
                RED, RESET, service
            );
        } else if verbose {
            println!("{}[SUCCESS]{} {} restarted.", GREEN, RESET, service);
        }
    }
    if !verbose {
        println!("🍺 System services restarted.");
    }
    Ok(())
}
