use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use toml::Value;

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

/// Returns the path for the snapshot file.
/// The snapshot stores the last-applied configuration.
pub fn get_snapshot_path() -> PathBuf {
    let mut snapshot_path = get_config_path();
    snapshot_path.set_file_name(".cutler_snapshot");
    snapshot_path
}

/// Helper: Read and parse the configuration file from a given path.
fn load_config(path: &PathBuf) -> Result<Value, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let parsed: Value = content.parse::<Value>()?;
    Ok(parsed)
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
            Some(x) => x.clone(),
            None => String::new(),
        };
        dest.push((domain, flat_table));
    }

    for (key, value) in nested_tables {
        if let Value::Table(inner) = value {
            let new_prefix = if let Some(ref p) = prefix {
                if p.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", p, key)
                }
            } else {
                key.clone()
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

/// Helper: Collect domains and their settings from a toml::Value.
/// Returns a HashMap where keys are "domain" strings and values are the flattened settings table.
fn collect_domains(
    parsed: &Value,
) -> Result<HashMap<String, toml::value::Table>, Box<dyn std::error::Error>> {
    let root_table = parsed
        .as_table()
        .ok_or("Invalid config format: expected table at top level")?;
    let mut domains = HashMap::new();
    for (key, value) in root_table {
        if let Value::Table(inner) = value {
            let mut flat: Vec<(String, toml::value::Table)> = Vec::new();
            flatten_domains(Some(key.clone()), inner, &mut flat);
            for (domain, table) in flat {
                domains.insert(domain, table);
            }
        }
    }
    Ok(domains)
}

/// Applies settings by comparing the current config against a snapshot (if one exists).
/// If no snapshot exists, all settings from the config file are applied.
/// If a snapshot exists, then:
///   • New domains are applied.
///   • Modified domains are re-applied (updated).
///   • Removed domains (present in the snapshot but not in the current config) are unapplied.
/// After applying changes, a new snapshot is stored.
pub fn apply_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    // If no config file found, offer to create an example.
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

    // Load and parse the current config.
    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;

    let snapshot_path = get_snapshot_path();
    let snapshot_exists = snapshot_path.exists();

    // If snapshot exists, load it and compare.
    let snapshot_domains = if snapshot_exists {
        let snap_parsed = load_config(&snapshot_path)?;
        collect_domains(&snap_parsed)?
    } else {
        HashMap::new()
    };

    // For protection, always check each domain's existence using defaults read.
    // Now decide what to do.
    if !snapshot_exists {
        if verbose {
            println!("No snapshot found – applying all settings.");
        }
        // Apply every domain in current_domains.
        for (domain, settings_table) in &current_domains {
            // For NSGlobalDomain support.
            let effective_domain = if domain.starts_with("NSGlobalDomain") {
                "NSGlobalDomain".to_string()
            } else {
                format!("com.apple.{}", domain)
            };
            check_domain_exists(&effective_domain)?;
            for (key, value) in settings_table {
                let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, key);
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
    } else {
        // Compare snapshot with current:
        let mut new_domains = Vec::new();
        let mut modified_domains = Vec::new();
        let mut removed_domains = Vec::new();

        // Determine new or modified domains.
        for (domain, current_table) in &current_domains {
            match snapshot_domains.get(domain) {
                None => new_domains.push(domain.clone()),
                Some(old_table) => {
                    if old_table != current_table {
                        modified_domains.push(domain.clone())
                    }
                }
            }
        }
        // Determine removed domains.
        for domain in snapshot_domains.keys() {
            if !current_domains.contains_key(domain) {
                removed_domains.push(domain.clone());
            }
        }

        if verbose {
            println!("Changes detected:");
            if !new_domains.is_empty() {
                println!("New domains: {:?}", new_domains);
            }
            if !modified_domains.is_empty() {
                println!("Modified domains: {:?}", modified_domains);
            }
            if !removed_domains.is_empty() {
                println!("Removed domains (to be unapplied): {:?}", removed_domains);
            }
        }

        // Apply new domains and update modified ones.
        for domain in new_domains.iter().chain(modified_domains.iter()) {
            let settings_table = current_domains.get(domain).unwrap();
            let effective_domain = if domain.starts_with("NSGlobalDomain") {
                "NSGlobalDomain".to_string()
            } else {
                format!("com.apple.{}", domain)
            };
            check_domain_exists(&effective_domain)?;
            for (key, value) in settings_table {
                let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, key);
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
                        "Applying/Updating: defaults write {} \"{}\" {} \"{}\"",
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
                        "{}[ERROR]{} Failed to apply/update setting '{}' for {}.",
                        RED, RESET, eff_key, eff_domain
                    );
                } else if verbose {
                    println!(
                        "{}[SUCCESS]{} Applied/Updated setting '{}' for {}.",
                        GREEN, RESET, eff_key, eff_domain
                    );
                }
            }
            if !verbose {
                println!("Updated {}", effective_domain);
            }
        }
        // Unapply domains that were removed from the config.
        for domain in removed_domains {
            // Use the snapshot for the settings.
            let settings_table = snapshot_domains.get(&domain).unwrap();
            let effective_domain = if domain.starts_with("NSGlobalDomain") {
                "NSGlobalDomain".to_string()
            } else {
                format!("com.apple.{}", domain)
            };
            check_domain_exists(&effective_domain)?;
            for (key, _value) in settings_table {
                let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, key);
                if verbose {
                    println!(
                        "Unapplying (removed): defaults delete {} \"{}\"",
                        eff_domain, eff_key
                    );
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
    }

    // Save the current config as the new snapshot.
    fs::copy(&config_path, &snapshot_path)?;
    if verbose {
        println!(
            "{}[SUCCESS]{} Snapshot updated at {:?}.",
            GREEN, RESET, snapshot_path
        );
    }

    Ok(())
}

/// Unapplies settings by using the stored snapshot for comparison.
/// If no snapshot exists, an error is returned.
/// If the current configuration differs from the snapshot, the user is warned and asked for confirmation.
pub fn unapply_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_path = get_snapshot_path();

    if !snapshot_path.exists() {
        return Err("No snapshot found. Please apply settings first before unapplying.".into());
    }

    // Load snapshot and current config.
    let snap_parsed = load_config(&snapshot_path)?;
    let snap_domains = collect_domains(&snap_parsed)?;

    let config_path = get_config_path();
    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;

    // Compare snapshot and current.
    if snap_domains != current_domains {
        println!("Warning: The snapshot (last applied) differs from the current configuration.");
        print!("Are you sure you want to unapply everything? [y/N]: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            return Err("Aborted unapply due to configuration differences.".into());
        }
    }

    // Unapply every domain based on the snapshot.
    for (domain, settings_table) in snap_domains {
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

    // Optionally remove the snapshot after unapplying.
    fs::remove_file(&snapshot_path)?;
    if verbose {
        println!(
            "{}[SUCCESS]{} Snapshot removed from {:?}.",
            GREEN, RESET, snapshot_path
        );
    }

    Ok(())
}

/// Deletes the configuration file.
/// Before deletion, it checks whether the defaults are still applied (via defaults read)
/// and asks if the user wants to unapply first.
pub fn delete_config(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if !config_path.exists() {
        if verbose {
            println!(
                "{}[SUCCESS]{} No configuration file found at: {:?}",
                GREEN, RESET, config_path
            );
        } else {
            println!("🍺 No config file to delete.");
        }
        return Ok(());
    }

    // Load current config and check domains.
    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;
    let mut applied_domains = Vec::new();
    for (domain, _) in current_domains {
        let effective_domain = if domain.starts_with("NSGlobalDomain") {
            "NSGlobalDomain".to_string()
        } else {
            format!("com.apple.{}", domain)
        };
        // We try to read the domain; if it exists, assume settings are applied.
        if Command::new("defaults")
            .arg("read")
            .arg(&effective_domain)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            applied_domains.push(effective_domain);
        }
    }

    if !applied_domains.is_empty() {
        println!(
            "The following domains appear to still be applied: {:?}",
            applied_domains
        );
        print!("Would you like to unapply these settings before deleting the config file? [y/N]: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            // Call unapply using our snapshot.
            // If no snapshot exists, unapply_defaults will refuse.
            unapply_defaults(verbose)?;
        }
    }

    fs::remove_file(&config_path)?;
    if verbose {
        println!(
            "{}[SUCCESS]{} Configuration file deleted from: {:?}",
            GREEN, RESET, config_path
        );
    } else {
        println!("🗑️ Config deleted from {:?}", config_path);
    }

    // Also remove snapshot if present.
    let snapshot_path = get_snapshot_path();
    if snapshot_path.exists() {
        fs::remove_file(&snapshot_path)?;
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
