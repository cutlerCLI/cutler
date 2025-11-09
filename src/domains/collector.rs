// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use defaults_rs::{Domain, Preferences};
use std::collections::HashMap;
use toml::{Table, Value};

/// Collect all tables in `[set]` and return a map domain → settings.
/// Note: TOML sections like [set.finder.FXInfoPanelsExpanded] are already parsed
/// as separate top-level domains by TOML, so we don't need recursive flattening.
/// Inline tables like FXInfoPanelsExpanded = { Preview = false } should be kept
/// as dictionary values, not flattened into sub-domains.
pub fn collect(config: &crate::config::core::Config) -> Result<HashMap<String, Table>> {
    let mut out = HashMap::new();

    if let Some(set) = &config.set {
        for (domain_key, domain_val) in set {
            // domain_val: HashMap<String, Value>
            // TOML already parsed section headers like [set.finder.nested] 
            // into separate entries, so domain_key is already the full domain name.
            // We just need to convert the HashMap to a Table.
            let mut table = Table::new();
            for (k, v) in domain_val {
                table.insert(k.clone(), v.clone());
            }
            out.insert(domain_key.clone(), table);
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
