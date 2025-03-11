// Imports.
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use toml::Value;

/// Color constants.
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RESET: &str = "\x1b[0m";

/// Log levels for printing messages.
#[derive(PartialEq)]
pub enum LogLevel {
    Success,
    Error,
    Warning,
    Info,
}

/// Central printing function.
pub fn print_log(level: LogLevel, message: &str, is_verbose: bool) {
    // Only print Info messages if verbose is on.
    if level == LogLevel::Info && !is_verbose {
        return;
    }
    match level {
        LogLevel::Success => println!("{}[SUCCESS]{} {}", GREEN, RESET, message),
        LogLevel::Error => eprintln!("{}[ERROR]{} {}", RED, RESET, message),
        LogLevel::Warning => eprintln!("{}[WARN]{} {}", YELLOW, RESET, message),
        LogLevel::Info => println!("{}", message),
    }
}

/// Returns the path to the config file, respecting XDG_CONFIG_HOME if available.
pub fn get_config_path() -> PathBuf {
    if let Some(xdg_config) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join("cutler").join("config.toml")
    } else if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("cutler")
            .join("config.toml")
    } else {
        // Fallback to a relative path if HOME is not set.
        PathBuf::from("config.toml")
    }
}

/// Returns the path for the snapshot file.
/// The snapshot stores the last-applied configuration.
pub fn get_snapshot_path() -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home).join(".cutler_snapshot")
    } else {
        PathBuf::from(".cutler_snapshot")
    }
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
        print_log(
            LogLevel::Success,
            &format!("Example config created at: {:?}", path),
            verbose,
        );
    } else {
        // For non-verbose, simply print a minimal message with an emoji.
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
            Some(s) => s.clone(),
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

/// Helper: Returns the flag and string representation for a given value.
/// For booleans, they are written as "true" or "false" strings.
fn get_flag_and_value(value: &Value) -> Result<(&'static str, String), Box<dyn std::error::Error>> {
    match value {
        Value::Boolean(b) => Ok(("-bool", if *b { "true".into() } else { "false".into() })),
        Value::Integer(_) => Ok(("-int", value.to_string())),
        Value::Float(_) => Ok(("-float", value.to_string())),
        Value::String(_) => Ok(("-string", value.as_str().unwrap().to_string())),
        _ => Err("Unsupported type encountered in configuration".into()),
    }
}

/// Helper: Executes a "defaults write" command with the provided parameters.
fn execute_defaults_write(
    eff_domain: &str,
    eff_key: &str,
    flag: &str,
    value_str: &str,
    action: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        print_log(
            LogLevel::Info,
            &format!(
                "{}: defaults write {} \"{}\" {} \"{}\"",
                action, eff_domain, eff_key, flag, value_str
            ),
            verbose,
        );
    }
    let output = Command::new("defaults")
        .arg("write")
        .arg(eff_domain)
        .arg(eff_key)
        .arg(flag)
        .arg(value_str)
        .output()?;
    if !output.status.success() {
        eprintln!(
            "{}[ERROR]{} Failed to {} setting '{}' for {}.",
            RED,
            RESET,
            action.to_lowercase(),
            eff_key,
            eff_domain
        );
    } else if verbose {
        print_log(
            LogLevel::Success,
            &format!("{} setting '{}' for {}.", action, eff_key, eff_domain),
            verbose,
        );
    }
    Ok(())
}

/// Helper: Executes a "defaults delete" command with the provided parameters.
fn execute_defaults_delete(
    eff_domain: &str,
    eff_key: &str,
    action: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        print_log(
            LogLevel::Info,
            &format!("{}: defaults delete {} \"{}\"", action, eff_domain, eff_key),
            verbose,
        );
    }
    let output = Command::new("defaults")
        .arg("delete")
        .arg(eff_domain)
        .arg(eff_key)
        .output()?;
    if !output.status.success() {
        eprintln!(
            "{}[ERROR]{} Failed to {} setting '{}' for {}.",
            RED,
            RESET,
            action.to_lowercase(),
            eff_key,
            eff_domain
        );
    } else if verbose {
        print_log(
            LogLevel::Success,
            &format!("{} setting '{}' for {}.", action, eff_key, eff_domain),
            verbose,
        );
    }
    Ok(())
}

/// Checks whether a given domain exists using `defaults read`.
pub fn check_domain_exists(full_domain: &str) -> Result<(), Box<dyn std::error::Error>> {
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

/// Helper: Read the current value from defaults (if any) for a given effective domain and key.
fn get_current_value(eff_domain: &str, eff_key: &str) -> Option<String> {
    let output = Command::new("defaults")
        .arg("read")
        .arg(eff_domain)
        .arg(eff_key)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Normalizes the desired value for comparison.
/// For booleans the normalized value is "1" for true and "0" for false.
/// For strings, returns the inner string without additional quotes.
fn normalize_desired(value: &Value) -> String {
    match value {
        Value::Boolean(b) => {
            if *b {
                "1".into()
            } else {
                "0".into()
            }
        }
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

/// Applies settings by reading current values via defaults read, and only executing
/// a defaults write if the current value does not match the desired value.
/// After changes are applied, the snapshot is updated with the current configuration.
pub fn apply_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        if verbose {
            print_log(
                LogLevel::Info,
                &format!("Config file not found at {:?}.", config_path),
                verbose,
            );
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

    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;

    for (domain, settings_table) in &current_domains {
        let effective_domain = if domain.starts_with("NSGlobalDomain") {
            "NSGlobalDomain".to_string()
        } else {
            format!("com.apple.{}", domain)
        };
        check_domain_exists(&effective_domain)?;
        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(domain, key);
            let desired = normalize_desired(value);
            if let Some(curr) = get_current_value(&eff_domain, &eff_key) {
                if curr == desired {
                    continue;
                }
            }
            let (flag, value_str) = get_flag_and_value(value)?;
            execute_defaults_write(&eff_domain, &eff_key, flag, &value_str, "Applying", verbose)?;
        }
    }

    let snapshot_path = get_snapshot_path();
    fs::copy(&config_path, &snapshot_path)?;
    print_log(
        LogLevel::Success,
        &format!("Snapshot updated at {:?}", snapshot_path),
        verbose,
    );
    Ok(())
}

/// Unapplies settings by using the stored snapshot for comparison.
pub fn unapply_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_path = get_snapshot_path();
    if !snapshot_path.exists() {
        return Err("No snapshot found. Please apply settings first before unapplying.".into());
    }

    let snap_parsed = load_config(&snapshot_path)?;
    let snap_domains = collect_domains(&snap_parsed)?;

    let config_path = get_config_path();
    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;

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

    for (domain, settings_table) in snap_domains {
        let effective_domain = if domain.starts_with("NSGlobalDomain") {
            "NSGlobalDomain".to_string()
        } else {
            format!("com.apple.{}", domain)
        };
        check_domain_exists(&effective_domain)?;
        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, &key);
            let desired = normalize_desired(&value);
            if let Some(curr) = get_current_value(&eff_domain, &eff_key) {
                if curr != desired {
                    print_log(
                        LogLevel::Warning,
                        &format!(
                            "{}.{} has been modified (expected {} but got {}). Skipping removal.",
                            eff_domain, eff_key, desired, curr
                        ),
                        true,
                    );
                    continue;
                }
            } else {
                continue;
            }
            execute_defaults_delete(&eff_domain, &eff_key, "Unapplying", verbose)?;
        }
    }

    fs::remove_file(&snapshot_path)?;
    print_log(
        LogLevel::Success,
        &format!("Snapshot removed from {:?}", snapshot_path),
        verbose,
    );
    Ok(())
}

/// Deletes the configuration file.
pub fn delete_config(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if !config_path.exists() {
        print_log(
            LogLevel::Success,
            &format!("No configuration file found at: {:?}", config_path),
            verbose,
        );
        return Ok(());
    }

    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;
    let mut applied_domains = Vec::new();
    for (domain, _) in current_domains {
        let effective_domain = if domain.starts_with("NSGlobalDomain") {
            "NSGlobalDomain".to_string()
        } else {
            format!("com.apple.{}", domain)
        };
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
            unapply_defaults(verbose)?;
        }
    }

    fs::remove_file(&config_path)?;
    print_log(
        LogLevel::Success,
        &format!("Configuration file deleted from: {:?}", config_path),
        verbose,
    );

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
            print_log(
                LogLevel::Success,
                &format!("{} restarted.", service),
                verbose,
            );
        }
    }
    if !verbose {
        println!("🍺 System services restarted.");
    }
    Ok(())
}
