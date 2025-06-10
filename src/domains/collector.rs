use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, Once, LazyLock};
use tokio::process::Command;
use toml::Value;

static DOMAIN_CACHE: LazyLock<Mutex<Option<HashSet<String>>>> = LazyLock::new(|| Mutex::new(None));
static INIT: Once = Once::new();

/// Recursively flatten nested TOML tables into (domain, settings‑table) pairs.
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
                Some(p) if !p.is_empty() => format!("{}.{}", p, k),
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
pub fn collect(parsed: &Value) -> Result<HashMap<String, toml::value::Table>, anyhow::Error> {
    let root = parsed
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("Config is not a TOML table"))?;
    let mut out = HashMap::new();

    for (key, val) in root {
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

/// Given a config‑domain and key, return the effective “defaults” domain + key.
pub fn effective(domain: &str, key: &str) -> (String, String) {
    if domain == "NSGlobalDomain" {
        ("NSGlobalDomain".into(), key.into())
    } else if let Some(rest) = domain.strip_prefix("NSGlobalDomain.") {
        if rest.is_empty() {
            ("NSGlobalDomain".into(), key.into())
        } else {
            ("NSGlobalDomain".into(), format!("{}.{}", rest, key))
        }
    } else {
        (format!("com.apple.{}", domain), key.into())
    }
}

/// do we need to prefix “com.apple.” on this domain?
pub fn needs_prefix(domain: &str) -> bool {
    !domain.starts_with("NSGlobalDomain")
}

/// Check whether a domain exists.
async fn domain_exists(full: &str) -> bool {
    {
        let cache = DOMAIN_CACHE.lock().unwrap();
        if let Some(set) = &*cache {
            if set.contains(full) {
                return true;
            }
        }
    }
    // direct read as fallback
    Command::new("defaults")
        .arg("read")
        .arg(full)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Extension of domain_exists() which also sets the cache.
/// Public check—errors out if the domain is missing.
pub async fn check_exists(full_domain: &str) -> Result<(), anyhow::Error> {
    INIT.call_once(|| {
        tokio::spawn(async {
            if let Ok(out) = Command::new("defaults").arg("domains").output().await {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout);
                    let set: HashSet<_> = s
                        .split(|c: char| c == ',' || c.is_whitespace())
                        .filter(|x| !x.is_empty())
                        .map(|x| x.to_string())
                        .collect();
                    *DOMAIN_CACHE.lock().unwrap() = Some(set);
                }
            }
        });
    });

    if domain_exists(full_domain).await {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Domain '{}' does not exist", full_domain))
    }
}

/// Read the current value of a defaults key, if any.
pub async fn read_current(eff_domain: &str, eff_key: &str) -> Option<String> {
    let out = Command::new("defaults")
        .arg("read")
        .arg(eff_domain)
        .arg(eff_key)
        .output()
        .await
        .ok()?;

    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}