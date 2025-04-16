use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{Mutex, Once};
use toml::Value;

use crate::logging::{LogLevel, print_log};

lazy_static! {
    static ref DOMAIN_CACHE: Mutex<Option<HashSet<String>>> = Mutex::new(None);
}

static INIT: Once = Once::new();

/// Recursively flattens a TOML table into a list of (domain, settings) pairs.
/// For example, [menuextra.clock] becomes ("menuextra.clock", { … }).
pub fn flatten_domains(
    prefix: Option<String>,
    table: &toml::value::Table,
    dest: &mut Vec<(String, toml::value::Table)>,
) {
    let mut flat_table = toml::value::Table::new();

    // Process all non-table values in one pass
    for (key, value) in table {
        if let Value::Table(inner) = value {
            // Create new prefix for nested table
            let new_prefix = match &prefix {
                Some(p) if !p.is_empty() => format!("{}.{}", p, key),
                _ => key.clone(),
            };

            // Process nested table recursively
            flatten_domains(Some(new_prefix), inner, dest);
        } else {
            // Add non-table values to flat table
            flat_table.insert(key.clone(), value.clone());
        }
    }

    // Only add if there are non-table values
    if !flat_table.is_empty() {
        dest.push((prefix.unwrap_or_default(), flat_table));
    }
}

/// Given the flattened domain (from config) and a key, return the effective domain and key.
///
/// • If the domain is not beginning with "NSGlobalDomain", returns ("com.apple.{domain}", key)
/// • For an entry exactly equal to "NSGlobalDomain", returns ("NSGlobalDomain", key)
/// • For an entry starting with "NSGlobalDomain.", returns ("NSGlobalDomain", "{rest-of-domain}.{key}")
pub fn get_effective_domain_and_key(domain: &str, key: &str) -> (String, String) {
    if domain == "NSGlobalDomain" {
        ("NSGlobalDomain".to_string(), key.to_string())
    } else if let Some(remainder) = domain.strip_prefix("NSGlobalDomain.") {
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

/// Gets the effective domain from a domain string
pub fn get_effective_domain(domain: &str) -> String {
    if domain.starts_with("NSGlobalDomain") {
        "NSGlobalDomain".to_string()
    } else {
        format!("com.apple.{}", domain)
    }
}

/// Collects domains and their flattened settings from a parsed TOML configuration.
/// Returns a HashMap where keys are domain strings and values are settings tables.
pub fn collect_domains(
    parsed: &toml::Value,
) -> Result<HashMap<String, toml::value::Table>, Box<dyn std::error::Error>> {
    let root_table = parsed
        .as_table()
        .ok_or("Invalid config format: expected table at top level")?;

    let mut domains = HashMap::new();

    for (key, value) in root_table {
        if key == "external" {
            continue;
        }

        if let Value::Table(inner) = value {
            // Pre-allocate with reasonable capacity to avoid resizes
            let mut flat = Vec::with_capacity(inner.len() * 2);
            flatten_domains(Some(key.clone()), inner, &mut flat);

            // Move items directly into the HashMap
            for (domain, table) in flat {
                domains.insert(domain, table);
            }
        }
    }

    Ok(domains)
}

/// Helper function to check if a domain exists
fn domain_exists(full_domain: &str) -> bool {
    let cache = DOMAIN_CACHE.lock().unwrap();
    if let Some(domains) = &*cache {
        domains.contains(full_domain)
    } else {
        // Fallback to direct check
        Command::new("defaults")
            .arg("read")
            .arg(full_domain)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Checks whether a given domain exists using the "defaults" command.
pub fn check_domain_exists(full_domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to initialize cache if needed
    INIT.call_once(|| {
        let output = match Command::new("defaults").arg("domains").output() {
            Ok(output) => output,
            Err(e) => {
                print_log(
                    LogLevel::Warning,
                    &format!(
                        "Failed to fetch domains: {}, some domain checks may fail",
                        e
                    ),
                );
                return; // Cache remains null, indicating that the user should be careful with domain checks
            }
        };

        if output.status.success() {
            let domains_str = String::from_utf8_lossy(&output.stdout);
            let domains: HashSet<String> = domains_str
                .split(|c: char| c == ',' || c.is_whitespace())
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect();

            let mut cache = DOMAIN_CACHE.lock().unwrap();
            *cache = Some(domains);
        }
    });

    if domain_exists(full_domain) {
        Ok(())
    } else {
        Err(format!("Domain '{}' does not exist. Aborting.", full_domain).into())
    }
}

/// Helper function to check if domain requires the com.apple prefix
pub fn needs_prefix(domain: &str) -> bool {
    !domain.starts_with("NSGlobalDomain")
}

/// Helper: Reads the current value from defaults (if any) for a given effective domain and key.
pub fn get_current_value(eff_domain: &str, eff_key: &str) -> Option<String> {
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
    if s.is_empty() { None } else { Some(s) }
}
