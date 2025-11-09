// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use defaults_rs::{Domain, Preferences};
use std::collections::HashMap;
use toml::{Table, Value};

/// Recursively flatten nested TOML tables that represent domain hierarchies.
/// Only flattens top-level nested tables (from section headers like [set.domain.subdomain]),
/// preserving inline table dictionary values.
fn flatten_domains(
    prefix: Option<String>,
    table: &toml::value::Table,
    dest: &mut Vec<(String, Table)>,
    depth: usize,
) {
    let mut flat = Table::new();

    for (k, v) in table {
        if let Value::Table(inner) = v {
            // Only flatten at depth 0 (top level after domain_key)
            // This handles [set.domain.subdomain] section nesting
            // but preserves inline tables like key = { x = 1 } as dictionary values
            if depth == 0 {
                let new_prefix = match &prefix {
                    Some(p) if !p.is_empty() => format!("{p}.{k}"),
                    _ => k.clone(),
                };
                flatten_domains(Some(new_prefix), inner, dest, depth + 1);
            } else {
                // Preserve as-is (it's an inline table value)
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
pub fn collect(config: &crate::config::core::Config) -> Result<HashMap<String, Table>> {
    let mut out = HashMap::new();

    if let Some(set) = &config.set {
        for (domain_key, domain_val) in set {
            // domain_val: HashMap<String, Value>
            let mut inner_table = Table::new();
            for (k, v) in domain_val {
                inner_table.insert(k.clone(), v.clone());
            }
            let mut flat = Vec::with_capacity(inner_table.len());
            flatten_domains(Some(domain_key.clone()), &inner_table, &mut flat, 0);

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
    let domain_obj = if eff_domain == "NSGlobalDomain" {
        Domain::Global
    } else if let Some(rest) = eff_domain.strip_prefix("com.apple.") {
        Domain::User(format!("com.apple.{rest}"))
    } else {
        Domain::User(eff_domain.to_string())
    };

    match Preferences::read(domain_obj, Some(eff_key)).await {
        Ok(result) => Some(result),
        Err(_) => None,
    }
}
