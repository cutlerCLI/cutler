use toml::Value;

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
/// • If the domain is not beginning with "NSGlobalDomain", returns ("com.apple.<domain>", key)
/// • For an entry exactly equal to "NSGlobalDomain", returns ("NSGlobalDomain", key)
/// • For an entry starting with "NSGlobalDomain.", returns ("NSGlobalDomain", "<rest-of-domain>.<key>")
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

/// Collects domains and their flattened settings from a parsed TOML configuration.
/// Returns a HashMap where keys are domain strings and values are settings tables.
pub fn collect_domains(
    parsed: &toml::Value,
) -> Result<std::collections::HashMap<String, toml::value::Table>, Box<dyn std::error::Error>> {
    let root_table = parsed
        .as_table()
        .ok_or("Invalid config format: expected table at top level")?;

    let mut domains = std::collections::HashMap::new();

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
