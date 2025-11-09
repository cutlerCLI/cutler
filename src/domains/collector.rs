// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use defaults_rs::{Domain, Preferences};
use std::collections::HashMap;
use toml::{Table, Value};

/// Convert a domain string to a Domain object.
/// Helper function to reduce code duplication.
pub fn domain_string_to_obj(domain: &str) -> Domain {
    if domain == "NSGlobalDomain" {
        Domain::Global
    } else {
        Domain::User(domain.to_string())
    }
}

/// Recursively flatten nested TOML tables that represent domain hierarchies.
/// Uses the actual domain list to distinguish between nested domains and inline dictionaries.
fn flatten_domains(
    prefix: Option<String>,
    table: &toml::value::Table,
    dest: &mut Vec<(String, Table)>,
    depth: usize,
    valid_domains: Option<&[String]>,
) {
    let mut flat = Table::new();

    for (k, v) in table {
        if let Value::Table(inner) = v {
            // At depth 0, check if this could be a valid domain
            if depth == 0 {
                let potential_domain = match &prefix {
                    Some(p) if !p.is_empty() => format!("{p}.{k}"),
                    _ => k.clone(),
                };
                
                // If we have a valid domains list, use it to check
                let should_flatten = if let Some(domains) = valid_domains {
                    // Convert to effective domain name to check
                    let effective_domain = get_defaults_domain(&potential_domain);
                    
                    // Only flatten if this would be a valid domain OR if it's NSGlobalDomain
                    effective_domain == "NSGlobalDomain" || domains.contains(&effective_domain)
                } else {
                    // No domain validation available (e.g., in tests), use old behavior:
                    // flatten at depth 0 only
                    true
                };
                
                if should_flatten {
                    flatten_domains(Some(potential_domain), inner, dest, depth + 1, valid_domains);
                } else {
                    // Not a valid domain, keep as inline table value
                    flat.insert(k.clone(), v.clone());
                }
            } else {
                // Preserve as-is (we're already past depth 0)
                flat.insert(k.clone(), v.clone());
            }
        } else {
            flat.insert(k.clone(), v.clone());
        }
    }

    if !flat.is_empty() {
        dest.push((prefix.unwrap_or_default(), flat));
    }
}

/// Collect all tables in `[set]` and return a map domain → settings.
/// Handles both section header nesting (e.g., [set.domain.subdomain]) and
/// inline table dictionary values (e.g., key = { x = 1, y = 2 }).
/// Uses the system's actual domain list to distinguish between domains and dictionaries.
pub async fn collect(config: &crate::config::core::Config) -> Result<HashMap<String, Table>> {
    let mut out = HashMap::new();

    if let Some(set) = &config.set {
        // Get the list of valid domains from the system
        let valid_domains: Option<Vec<String>> = Preferences::list_domains()
            .await
            .ok()
            .map(|domains| domains.iter().map(|d| d.to_string()).collect());

        for (domain_key, domain_val) in set {
            // domain_val: HashMap<String, Value>
            let mut inner_table = Table::new();
            for (k, v) in domain_val {
                inner_table.insert(k.clone(), v.clone());
            }
            let mut flat = Vec::with_capacity(inner_table.len());
            flatten_domains(
                Some(domain_key.clone()), 
                &inner_table, 
                &mut flat, 
                0, 
                valid_domains.as_deref()
            );

            for (domain, tbl) in flat {
                out.insert(domain, tbl);
            }
        }
    }
    Ok(out)
}

/// Helper for: effective()
/// Turn a config‐domain into the real defaults domain.
///   finder            -> com.apple.finder
///   NSGlobalDomain    -> NSGlobalDomain
///   NSGlobalDomain.bar-> NSGlobalDomain
fn get_defaults_domain(domain: &str) -> String {
    if domain.strip_prefix("NSGlobalDomain.").is_some() {
        // NSGlobalDomain.foo -> NSGlobalDomain
        "NSGlobalDomain".into()
    } else if domain == "NSGlobalDomain" {
        domain.into()
    } else {
        // anything else gets com.apple.
        format!("com.apple.{domain}")
    }
}

/// Given the TOML domain and key, figure out the true domain-key pair.
pub fn effective(domain: &str, key: &str) -> (String, String) {
    let dom = get_defaults_domain(domain);
    let k = if dom == "NSGlobalDomain" && domain.starts_with("NSGlobalDomain.") {
        // NSGlobalDomain.foo + key  -> foo.key
        let rest = &domain["NSGlobalDomain.".len()..];
        format!("{rest}.{key}")
    } else {
        key.into()
    };
    (dom, k)
}

/// Read the current value of a defaults key, if any.
pub async fn read_current(eff_domain: &str, eff_key: &str) -> Option<defaults_rs::PrefValue> {
    let domain_obj = domain_string_to_obj(eff_domain);

    match Preferences::read(domain_obj, Some(eff_key)).await {
        Ok(result) => Some(result),
        Err(_) => None,
    }
}
