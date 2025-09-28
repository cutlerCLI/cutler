// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, bail};
use defaults_rs::PrefValue;
use std::collections::HashMap;
use toml::Value;

/// Turns a toml::Value into its defaults_rs::PrefValue counterpart.
pub fn toml_to_prefvalue(val: &Value) -> anyhow::Result<PrefValue> {
    Ok(match val {
        Value::String(s) => PrefValue::String(s.clone()),
        Value::Integer(i) => PrefValue::Integer(*i),
        Value::Float(f) => PrefValue::Float(*f),
        Value::Boolean(b) => PrefValue::Boolean(*b),
        Value::Array(arr) => PrefValue::Array(
            arr.iter()
                .map(toml_to_prefvalue)
                .collect::<Result<Vec<_>>>()?,
        ),
        Value::Table(tbl) => PrefValue::Dictionary(
            tbl.iter()
                .map(|(k, v)| Ok((k.clone(), toml_to_prefvalue(v)?)))
                .collect::<Result<HashMap<_, _>>>()?,
        ),
        _ => bail!("Unsupported TOML value for PrefValue"),
    })
}

/// Turns a defaults_rs::PrefValue into its toml::Value counterpart.
pub fn prefvalue_to_toml(val: &PrefValue) -> Value {
    match val {
        PrefValue::String(s) => Value::String(s.clone()),
        PrefValue::Integer(i) => Value::Integer(*i),
        PrefValue::Float(f) => Value::Float(*f),
        PrefValue::Boolean(b) => Value::Boolean(*b),
        PrefValue::Array(arr) => Value::Array(arr.iter().map(prefvalue_to_toml).collect()),
        PrefValue::Dictionary(dict) => {
            let map = dict
                .iter()
                .map(|(k, v)| (k.clone(), prefvalue_to_toml(v)))
                .collect();
            Value::Table(map)
        }
    }
}

/// Turns a string into its toml::Value counterpart.
pub fn string_to_toml_value(s: &str) -> toml::Value {
    // try bool, int, float, fallback to string
    if s == "true" {
        toml::Value::Boolean(true)
    } else if s == "false" {
        toml::Value::Boolean(false)
    } else if let Ok(i) = s.parse::<i64>() {
        toml::Value::Integer(i)
    } else if let Ok(f) = s.parse::<f64>() {
        toml::Value::Float(f)
    } else {
        toml::Value::String(s.to_string())
    }
}

/// Turns a PrefValue object to a string.
pub fn prefvalue_to_string(val: &PrefValue) -> String {
    match val {
        PrefValue::Boolean(b) => b.to_string(),
        PrefValue::Integer(i) => i.to_string(),
        PrefValue::Float(f) => f.to_string(),
        PrefValue::String(s) => s.clone(),
        PrefValue::Array(arr) => {
            let inner = arr
                .iter()
                .map(prefvalue_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{inner}]")
        }
        PrefValue::Dictionary(dict) => {
            let inner = dict
                .iter()
                .map(|(k, v)| format!("{}: {}", k, prefvalue_to_string(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{inner}}}")
        }
    }
}

/// Normalize a toml::Value to a string.
pub fn normalize(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}
