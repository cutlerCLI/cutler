use crate::logging::{LogLevel, print_log};
use std::env;
use std::process::Command;
use toml::Value;

fn substitute_arg(arg: &str, variables: Option<&toml::value::Table>, result: &mut Vec<String>) {
    if !arg.starts_with('$') {
        // No substitution needed
        result.push(arg.to_owned());
        return;
    }

    // Extract key
    let key = if arg.starts_with("${") && arg.ends_with('}') {
        &arg[2..arg.len() - 1]
    } else {
        &arg[1..]
    };

    // Check variables
    if let Some(vars) = variables {
        if let Some(var_value) = vars.get(key) {
            if let Some(arr) = var_value.as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        result.push(s.to_owned());
                    }
                }
                return;
            } else if let Some(s) = var_value.as_str() {
                result.push(s.to_owned());
                return;
            } else {
                result.push(var_value.to_string());
                return;
            }
        }
    }

    // Environment fallback
    if let Ok(env_val) = env::var(key) {
        result.push(env_val);
        return;
    }

    // No substitution found, use original
    result.push(arg.to_owned());
}

/// Executes external commands as before, with proper argument handling and variable support.
/// Looks for an optional [external.variables] table and uses it to substitute placeholders
/// in command arguments.
pub fn execute_external_commands(
    config: &Value,
    verbose: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Early return if no external section exists
    let Some(ext_section) = config.get("external") else {
        return Ok(());
    };

    let variables = ext_section.get("variables").and_then(|v| v.as_table());

    let Some(commands_array) = ext_section.get("command").and_then(|v| v.as_array()) else {
        return Ok(());
    };

    for command_val in commands_array {
        // Each command should be a table.
        let command_table = command_val
            .as_table()
            .ok_or("Invalid external command format: expected a table")?;

        // Get the command string (required).
        let cmd = command_table
            .get("cmd")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'cmd' in external command")?;

        // Process and substitute arguments.
        let final_args: Vec<String> = if let Some(arg_val) = command_table.get("args") {
            // Expect args to be an array.
            if let Some(arr) = arg_val.as_array() {
                let mut args_consolidated = Vec::new();
                for elem in arr {
                    if let Some(arg_str) = elem.as_str() {
                        let mut substituted = Vec::new();
                        substitute_arg(arg_str, variables, &mut substituted);
                        args_consolidated.extend(substituted);
                    } else {
                        return Err(format!(
                            "Non-string argument found in args for command '{}'",
                            cmd
                        )
                        .into());
                    }
                }
                args_consolidated
            } else if let Some(single_arg) = arg_val.as_str() {
                // Optionally, if a single string is provided, split it on whitespace.
                single_arg
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                return Err(
                    format!("Invalid type for 'args' in external command '{}'", cmd).into(),
                );
            }
        } else {
            Vec::new()
        };

        // Get the sudo flag; default to false.
        let sudo = command_table
            .get("sudo")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // If sudo is enabled, run "sudo" with the command and its arguments.
        let (exec_cmd, exec_args) = if sudo {
            ("sudo".to_string(), {
                let mut v = vec![cmd.to_string()];
                v.extend(final_args.clone());
                v
            })
        } else {
            (cmd.to_string(), final_args.clone())
        };

        if dry_run {
            print_log(
                LogLevel::Info,
                &format!(
                    "Dry-run: Would execute external command: {} {:?}",
                    exec_cmd, exec_args
                ),
            );
        } else {
            if verbose {
                print_log(
                    LogLevel::Info,
                    &format!("Executing external command: {} {:?}", exec_cmd, exec_args),
                );
            }
            let output = Command::new(&exec_cmd).args(&exec_args).output()?;

            if !output.status.success() {
                print_log(
                    LogLevel::Error,
                    &format!(
                        "External command failed: {} {:?}: {}",
                        exec_cmd,
                        exec_args,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                );
            } else if verbose {
                print_log(
                    LogLevel::Success,
                    &format!(
                        "External command executed: {} {:?}\nCommand output: {}",
                        exec_cmd,
                        exec_args,
                        String::from_utf8_lossy(&output.stdout)
                    ),
                );
            }
        }
    }

    Ok(())
}
