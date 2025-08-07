#[cfg(feature = "macos-deps")]
use defaults_rs::PrefValue;
use toml::Value;

/// Turns a toml::Value into its defaults_rs::PrefValue counterpart.
#[cfg(feature = "macos-deps")]
pub fn toml_to_prefvalue(val: &Value) -> anyhow::Result<PrefValue> {
    use std::collections::HashMap;
    
    Ok(match val {
        Value::String(s) => PrefValue::String(s.clone()),
        Value::Integer(i) => PrefValue::Integer(*i),
        Value::Float(f) => PrefValue::Float(*f),
        Value::Boolean(b) => PrefValue::Boolean(*b),
        Value::Array(arr) => PrefValue::Array(
            arr.iter()
                .map(toml_to_prefvalue)
                .collect::<Result<Vec<_>, anyhow::Error>>()?,
        ),
        Value::Table(tbl) => PrefValue::Dictionary(
            tbl.iter()
                .map(|(k, v)| Ok((k.clone(), toml_to_prefvalue(v)?)))
                .collect::<Result<HashMap<_, _>, anyhow::Error>>()?,
        ),
        _ => anyhow::bail!("Unsupported TOML value for PrefValue"),
    })
}

/// Turns a defaults_rs::PrefValue into its toml::Value counterpart.
#[cfg(feature = "macos-deps")]
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
    // Handle empty string edge case
    if s.is_empty() {
        return toml::Value::String(String::new());
    }
    
    // try bool, int, float, fallback to string
    if s == "true" {
        toml::Value::Boolean(true)
    } else if s == "false" {
        toml::Value::Boolean(false)
    } else if let Ok(i) = s.parse::<i64>() {
        toml::Value::Integer(i)
    } else if let Ok(f) = s.parse::<f64>() {
        // Handle NaN and infinity edge cases
        if f.is_nan() || f.is_infinite() {
            toml::Value::String(s.to_string())
        } else {
            toml::Value::Float(f)
        }
    } else {
        toml::Value::String(s.to_string())
    }
}

/// Normalize a toml::Value to a string.
pub fn normalize(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_toml_value_edge_cases() {
        // Test empty string
        assert_eq!(string_to_toml_value(""), toml::Value::String(String::new()));
        
        // Test valid numbers
        assert_eq!(string_to_toml_value("42"), toml::Value::Integer(42));
        assert_eq!(string_to_toml_value("3.14"), toml::Value::Float(3.14));
        
        // Test boolean values
        assert_eq!(string_to_toml_value("true"), toml::Value::Boolean(true));
        assert_eq!(string_to_toml_value("false"), toml::Value::Boolean(false));
        
        // Test NaN and infinity handling
        if let toml::Value::String(s) = string_to_toml_value("NaN") {
            assert_eq!(s, "NaN");
        } else {
            panic!("NaN should be treated as string");
        }
        
        if let toml::Value::String(s) = string_to_toml_value("inf") {
            assert_eq!(s, "inf");
        } else {
            panic!("inf should be treated as string");
        }
        
        // Test normal string
        assert_eq!(string_to_toml_value("hello"), toml::Value::String("hello".to_string()));
    }

    #[test]
    fn test_normalize_edge_cases() {
        // Test empty string
        let empty_value = toml::Value::String(String::new());
        assert_eq!(normalize(&empty_value), "");
        
        // Test various types
        let int_value = toml::Value::Integer(42);
        assert_eq!(normalize(&int_value), "42");
        
        let bool_value = toml::Value::Boolean(true);
        assert_eq!(normalize(&bool_value), "true");
        
        let float_value = toml::Value::Float(3.14);
        assert_eq!(normalize(&float_value), "3.14");
    }
}
