// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Result, bail};
use defaults_rs::PrefValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::Value;
use toml_edit::Value as EditValue;

/// Serializable representation of a preference value.
/// This mirrors the structure of `defaults_rs::PrefValue` but implements Serialize/Deserialize.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum SerializablePrefValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<SerializablePrefValue>),
    Dictionary(HashMap<String, SerializablePrefValue>),
}

/// Turns a `toml::Value` into its `defaults_rs::PrefValue` counterpart.
pub fn toml_to_prefvalue(val: &Value) -> Result<PrefValue> {
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

/// Turns a `defaults_rs::PrefValue` into its `toml::Value` counterpart.
pub fn prefvalue_to_toml(val: &PrefValue) -> Result<Value> {
    Ok(match val {
        PrefValue::String(s) => Value::String(s.clone()),
        PrefValue::Integer(i) => Value::Integer(*i),
        PrefValue::Float(f) => Value::Float(*f),
        PrefValue::Boolean(b) => Value::Boolean(*b),
        PrefValue::Array(arr) => Value::Array(
            arr.iter()
                .map(prefvalue_to_toml)
                .collect::<Result<Vec<_>>>()?,
        ),
        PrefValue::Dictionary(dict) => dict
            .iter()
            .map(|(k, v)| Ok((k.clone(), prefvalue_to_toml(v)?)))
            .collect::<Result<toml::map::Map<_, _>>>()
            .map(Value::Table)?,
        _ => bail!("Support does not extend to complex types of data."),
    })
}

/// Turns a string into its `toml::Value` counterpart.
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

/// Normalize a `toml::Value` to a string.
pub fn normalize(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

/// Turns a `toml_edit::Value` into its `defaults_rs::PrefValue` counterpart.
pub fn toml_edit_to_prefvalue(val: &EditValue) -> anyhow::Result<PrefValue> {
    Ok(match val {
        EditValue::String(s) => PrefValue::String(s.value().clone()),
        EditValue::Integer(i) => PrefValue::Integer(i.value().to_owned()),
        EditValue::Float(f) => PrefValue::Float(f.value().to_owned()),
        EditValue::Boolean(b) => PrefValue::Boolean(b.value().to_owned()),
        EditValue::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                result.push(toml_edit_to_prefvalue(item)?);
            }
            PrefValue::Array(result)
        }
        EditValue::InlineTable(tbl) => {
            let mut dict = HashMap::new();
            for (k, v) in tbl {
                dict.insert(k.to_string(), toml_edit_to_prefvalue(v)?);
            }
            PrefValue::Dictionary(dict)
        }
        _ => bail!("Unsupported toml_edit value type for PrefValue"),
    })
}

/// Converts a `toml_edit::Value` to a `toml::Value` for compatibility.
pub fn toml_edit_to_toml(val: &EditValue) -> anyhow::Result<Value> {
    Ok(match val {
        EditValue::String(s) => Value::String(s.value().clone()),
        EditValue::Integer(i) => Value::Integer(*i.value()),
        EditValue::Float(f) => Value::Float(*f.value()),
        EditValue::Boolean(b) => Value::Boolean(*b.value()),
        EditValue::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                result.push(toml_edit_to_toml(item)?);
            }
            Value::Array(result)
        }
        EditValue::InlineTable(tbl) => {
            let mut map = toml::map::Map::new();
            for (k, v) in tbl {
                map.insert(k.to_string(), toml_edit_to_toml(v)?);
            }
            Value::Table(map)
        }
        _ => bail!("Unsupported toml_edit value type"),
    })
}

/// Converts a `PrefValue` to a `SerializablePrefValue`.
/// Converts a `PrefValue` to a `SerializablePrefValue`.
pub fn prefvalue_to_serializable(val: &PrefValue) -> Result<SerializablePrefValue> {
    Ok(match val {
        PrefValue::String(s) => SerializablePrefValue::String(s.clone()),
        PrefValue::Integer(i) => SerializablePrefValue::Integer(*i),
        PrefValue::Float(f) => SerializablePrefValue::Float(*f),
        PrefValue::Boolean(b) => SerializablePrefValue::Boolean(*b),
        PrefValue::Array(arr) => SerializablePrefValue::Array(
            arr.iter()
                .map(prefvalue_to_serializable)
                .collect::<Result<Vec<_>>>()?,
        ),
        PrefValue::Dictionary(dict) => SerializablePrefValue::Dictionary(
            dict.iter()
                .map(|(k, v)| Ok((k.clone(), prefvalue_to_serializable(v)?)))
                .collect::<Result<HashMap<_, _>>>()?,
        ),
        _ => bail!("Unsupported PrefValue type"),
    })
}

/// Converts a `SerializablePrefValue` to a `PrefValue`.
pub fn serializable_to_prefvalue(val: &SerializablePrefValue) -> PrefValue {
    match val {
        SerializablePrefValue::String(s) => PrefValue::String(s.clone()),
        SerializablePrefValue::Integer(i) => PrefValue::Integer(*i),
        SerializablePrefValue::Float(f) => PrefValue::Float(*f),
        SerializablePrefValue::Boolean(b) => PrefValue::Boolean(*b),
        SerializablePrefValue::Array(arr) => {
            PrefValue::Array(arr.iter().map(serializable_to_prefvalue).collect())
        }
        SerializablePrefValue::Dictionary(dict) => PrefValue::Dictionary(
            dict.iter()
                .map(|(k, v)| (k.clone(), serializable_to_prefvalue(v)))
                .collect(),
        ),
    }
}
