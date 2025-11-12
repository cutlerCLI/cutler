// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use defaults_rs::{Domain, PrefValue, Preferences};
use std::collections::HashMap;
use toml::Table;

/// Collect all tables in `[set]`, parse with toml_edit to properly handle inline tables,
/// and return a map domain → settings.
pub fn collect(config: &crate::config::core::Config) -> Result<HashMap<String, Table>> {
    use crate::domains::convert::toml_edit_to_toml;
    use std::fs;
    use toml_edit::{DocumentMut, Item};

    let mut out = HashMap::new();

    // If we have the config path, read the raw file to parse with toml_edit
    // This allows us to distinguish inline tables from nested tables
    if !config.path.as_os_str().is_empty() && config.path.exists() {
        let content = fs::read_to_string(&config.path)?;
        let doc = content.parse::<DocumentMut>()?;

        if let Some(Item::Table(set_table)) = doc.get("set") {
            for (domain_key, item) in set_table.iter() {
                if let Item::Table(domain_table) = item {
                    // Now process the domain_table, checking if values are inline tables
                    let mut settings = Table::new();

                    for (key, value) in domain_table.iter() {
                        match value {
                            Item::Value(v) => {
                                // This could be a scalar value or an inline table
                                settings.insert(key.to_string(), toml_edit_to_toml(v)?);
                            }
                            Item::Table(nested_table) => {
                                // This is a nested table header [set.domain.nested]
                                // Recursively process it with the prefixed domain name
                                let nested_domain = format!("{}.{}", domain_key, key);
                                collect_nested_table(&nested_domain, nested_table, &mut out)?;
                            }
                            _ => {}
                        }
                    }

                    if !settings.is_empty() {
                        out.insert(domain_key.to_string(), settings);
                    }
                }
            }
        }
    } else {
        // Fallback: use the already-deserialized config.set
        // This is for tests or when config.path is not available
        if let Some(set) = &config.set {
            for (domain_key, domain_val) in set {
                let mut settings = Table::new();
                for (k, v) in domain_val {
                    settings.insert(k.clone(), v.clone());
                }
                if !settings.is_empty() {
                    out.insert(domain_key.clone(), settings);
                }
            }
        }
    }

    Ok(out)
}

/// Helper to recursively process nested tables
fn collect_nested_table(
    domain_prefix: &str,
    table: &toml_edit::Table,
    out: &mut HashMap<String, Table>,
) -> Result<()> {
    use crate::domains::convert::toml_edit_to_toml;
    use toml_edit::Item;

    let mut settings = Table::new();

    for (key, value) in table.iter() {
        match value {
            Item::Value(v) => {
                settings.insert(key.to_string(), toml_edit_to_toml(v)?);
            }
            Item::Table(nested_table) => {
                // Further nested table
                let nested_domain = format!("{}.{}", domain_prefix, key);
                collect_nested_table(&nested_domain, nested_table, out)?;
            }
            _ => {}
        }
    }

    if !settings.is_empty() {
        out.insert(domain_prefix.to_string(), settings);
    }

    Ok(())
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
pub async fn read_current(eff_domain: &str, eff_key: &str) -> Option<PrefValue> {
    let domain_obj = if eff_domain == "NSGlobalDomain" {
        Domain::Global
    } else if let Some(rest) = eff_domain.strip_prefix("com.apple.") {
        Domain::User(format!("com.apple.{rest}"))
    } else {
        Domain::User(eff_domain.to_string())
    };

    (Preferences::read(domain_obj, eff_key)).ok()
}
