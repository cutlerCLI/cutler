// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use defaults_rs::{Domain, ReadResult, preferences::Preferences};
use std::collections::HashMap;
use toml::{Table, Value};

use crate::domains::convert::prefvalue_to_string;

/// Recursively flatten nested TOML tables into (domain, settings-table) pairs.
fn flatten_domains(
    prefix: Option<String>,
    table: &toml::value::Table,
    dest: &mut Vec<(String, toml::value::Table)>,
) {
    let mut flat = toml::value::Table::new();

    for (k, v) in table {
        if let Value::Table(inner) = v {
            // descend into nested table
            let new_prefix = match &prefix {
                Some(p) if !p.is_empty() => format!("{p}.{k}"),
                _ => k.clone(),
            };
            flatten_domains(Some(new_prefix), inner, dest);
        } else {
            flat.insert(k.clone(), v.clone());
        }
    }

    if !flat.is_empty() {
        dest.push((prefix.unwrap_or_default(), flat));
    }
}

/// Collect all tables in `[set]`, flatten them, and return a map domain → settings.
pub fn collect(parsed: &Table) -> Result<HashMap<String, toml::value::Table>> {
    let mut out = HashMap::new();

    for (key, val) in parsed {
        if key == "set" {
            if let Value::Table(set_inner) = val {
                for (domain_key, domain_val) in set_inner {
                    if let Value::Table(inner) = domain_val {
                        let mut flat = Vec::with_capacity(inner.len());

                        flatten_domains(Some(domain_key.clone()), inner, &mut flat);

                        for (domain, tbl) in flat {
                            out.insert(domain, tbl);
                        }
                    }
                }
            }
            continue;
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
pub async fn read_current(eff_domain: &str, eff_key: &str) -> Option<String> {
    let domain_obj = if eff_domain == "NSGlobalDomain" {
        Domain::Global
    } else if let Some(rest) = eff_domain.strip_prefix("com.apple.") {
        Domain::User(format!("com.apple.{rest}"))
    } else {
        Domain::User(eff_domain.to_string())
    };

    match Preferences::read(domain_obj, Some(eff_key)).await {
        Ok(result) => match result {
            ReadResult::Value(val) => Some(prefvalue_to_string(&val)),
            ReadResult::Plist(plist_val) => Some(format!("{plist_val:?}")),
        },
        Err(_) => None,
    }
}
