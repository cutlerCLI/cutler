use toml::Value;

use crate::logging::{LogLevel, print_log};

/// For a given TOML value, returns the flag and string. Booleans become "-bool" with "true"/"false".
pub fn get_flag_and_value(
    value: &Value,
) -> Result<(&'static str, String), Box<dyn std::error::Error>> {
    match value {
        Value::Boolean(b) => Ok(("-bool", if *b { "true".into() } else { "false".into() })),
        Value::Integer(_) => Ok(("-int", value.to_string())),
        Value::Float(_) => Ok(("-float", value.to_string())),
        Value::String(s) => {
            // Using the value directly; no unwrap required.
            Ok(("-string", s.clone()))
        }
        _ => Err(format!("Unsupported type encountered in configuration: {:?}", value).into()),
    }
}

/// Executes a defaults command with the given parameters and handles logging.
fn execute_defaults_command(
    command: &str,
    eff_domain: &str,
    eff_key: &str,
    extra_args: Vec<&str>,
    action: &str,
    verbose: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd_display = format!("defaults {} {} \"{}\"", command, eff_domain, eff_key);
    for arg in &extra_args {
        cmd_display.push_str(&format!(" \"{}\"", arg));
    }

    if dry_run {
        print_log(
            LogLevel::Info,
            &format!("Dry-run: Would execute: {}", cmd_display),
        );
        return Ok(());
    }

    if verbose {
        print_log(LogLevel::Info, &format!("{}: {}", action, cmd_display));
    }

    let mut cmd = std::process::Command::new("defaults");
    cmd.arg(command).arg(eff_domain).arg(eff_key);

    for arg in extra_args {
        cmd.arg(arg);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        print_log(
            LogLevel::Error,
            &format!(
                "Failed to {} setting '{}' for {}.",
                action.to_lowercase(),
                eff_key,
                eff_domain
            ),
        );
    } else if verbose {
        print_log(
            LogLevel::Success,
            &format!("{} setting '{}' for {}.", action, eff_key, eff_domain),
        );
    }

    Ok(())
}

/// Executes a "defaults write" command with the given parameters.
pub fn execute_defaults_write(
    eff_domain: &str,
    eff_key: &str,
    flag: &str,
    value_str: &str,
    action: &str,
    verbose: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    execute_defaults_command(
        "write",
        eff_domain,
        eff_key,
        vec![flag, value_str],
        action,
        verbose,
        dry_run,
    )
}

/// Executes a "defaults delete" command with the specified parameters.
pub fn execute_defaults_delete(
    eff_domain: &str,
    eff_key: &str,
    action: &str,
    verbose: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    execute_defaults_command(
        "delete",
        eff_domain,
        eff_key,
        vec![],
        action,
        verbose,
        dry_run,
    )
}

/// Normalizes the desired value for comparison.
/// For booleans: maps true → "1" and false → "0". For strings, simply returns the inner string.
pub fn normalize_desired(value: &Value) -> String {
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
