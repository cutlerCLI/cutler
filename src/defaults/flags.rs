use toml::Value;

pub fn to_flag(value: &Value) -> anyhow::Result<(&'static str, String)> {
    match value {
        Value::Boolean(b) => Ok(("-bool", if *b { "true".into() } else { "false".into() })),
        Value::Integer(_) => Ok(("-int", value.to_string())),
        Value::Float(_) => Ok(("-float", value.to_string())),
        Value::String(s) => {
            // use value directly
            Ok(("-string", s.clone()))
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported type encountered in configuration: {:?}",
            value
        )),
    }
}

pub fn from_flag(value: &str) -> anyhow::Result<(&'static str, String)> {
    // bool
    if value == "1" || value == "0" || value == "true" || value == "false" {
        return Ok(("-bool", value.to_string()));
    }

    // integer
    if value.parse::<i64>().is_ok() {
        return Ok(("-int", value.to_string()));
    }

    // float
    if value.parse::<f64>().is_ok() {
        return Ok(("-float", value.to_string()));
    }

    // Default to the string type
    Ok(("-string", value.to_string()))
}

pub fn normalize(value: &Value) -> String {
    match value {
        Value::Boolean(b) => {
            if *b {
                "1".into()
            } else {
                "0".into()
            }
        }
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}
