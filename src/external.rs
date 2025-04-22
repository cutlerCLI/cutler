use crate::logging::{LogLevel, print_log};
use crate::snapshot::ExternalCommandState;
use std::env;
use std::process::{Command, Stdio};
use toml::Value;

/// Extracts external commands from the config into ExternalCommandState objects
pub fn extract_external_commands(config: &Value) -> Vec<ExternalCommandState> {
    let mut commands = Vec::new();

    if let Some(ext_section) = config.get("external") {
        if let Some(commands_array) = ext_section.get("command").and_then(|v| v.as_array()) {
            for command_val in commands_array {
                if let Some(command_table) = command_val.as_table() {
                    if let Some(cmd) = command_table.get("cmd").and_then(|v| v.as_str()) {
                        let args: Vec<String> = if let Some(arg_val) = command_table.get("args") {
                            if let Some(arr) = arg_val.as_array() {
                                arr.iter()
                                    .filter_map(|a| a.as_str())
                                    .map(String::from)
                                    .collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };

                        let sudo = command_table
                            .get("sudo")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        commands.push(ExternalCommandState {
                            cmd: cmd.to_string(),
                            args,
                            sudo,
                        });
                    }
                }
            }
        }
    }

    commands
}

/// Substitutes variables in a string from both custom variables and environment
fn substitute_variables(text: &str, variables: Option<&toml::value::Table>) -> String {
    let mut result = text.to_string();
    let mut var_positions = Vec::new();

    // Find all variable references
    let mut i = 0;
    while i < result.len() {
        if result[i..].starts_with('$') {
            let start = i;
            i += 1;

            // Handle ${var} format
            let is_braced = i < result.len() && result[i..].starts_with('{');
            if is_braced {
                i += 1;
                while i < result.len() && result.chars().nth(i) != Some('}') {
                    i += 1;
                }
                if i < result.len() {
                    i += 1; // Include closing brace
                }
            } else {
                // Handle $var format - variable name can include alphanumeric and underscore
                while i < result.len()
                    && result
                        .chars()
                        .nth(i)
                        .is_some_and(|c| c.is_alphanumeric() || c == '_')
                {
                    i += 1;
                }
            }

            var_positions.push((start, i));
        } else {
            i += 1;
        }
    }

    // Process variables from end to start to avoid position shifts
    for (start, end) in var_positions.into_iter().rev() {
        let var_ref = &result[start..end];

        // Extract variable name
        let var_name = if var_ref.starts_with("${") && var_ref.ends_with('}') {
            &var_ref[2..var_ref.len() - 1]
        } else {
            &var_ref[1..]
        };

        // Try to find value in custom variables
        let replacement = if let Some(vars) = variables {
            if let Some(value) = vars.get(var_name) {
                match value {
                    Value::String(s) => Some(s.clone()),
                    Value::Array(arr) => {
                        let joined = arr
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(" ");
                        Some(joined)
                    }
                    _ => Some(value.to_string()),
                }
            } else {
                None
            }
        } else {
            None
        };

        // If not found in custom variables, try environment
        let final_replacement = match replacement {
            Some(val) => val,
            None => env::var(var_name).unwrap_or_else(|_| var_ref.to_string()),
        };

        // Replace in the result string
        result.replace_range(start..end, &final_replacement);
    }

    result
}

/// Executes external commands, providing unified shell execution with robust variable handling
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

        let sudo = command_table
            .get("sudo")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Determine the full command
        let full_command = if let Some(arg_val) = command_table.get("args") {
            // Build command with args array
            let mut command_str = cmd.to_string();

            // Process args array
            if let Some(arr) = arg_val.as_array() {
                for arg in arr {
                    if let Some(arg_str) = arg.as_str() {
                        // Substitute variables in each argument
                        let processed_arg = substitute_variables(arg_str, variables);

                        // Quote argument if needed and append to command
                        if processed_arg.contains(' ')
                            && !(processed_arg.starts_with('"') && processed_arg.ends_with('"'))
                            && !(processed_arg.starts_with('\'') && processed_arg.ends_with('\''))
                        {
                            command_str.push_str(&format!(" \"{}\"", processed_arg));
                        } else {
                            command_str.push_str(&format!(" {}", processed_arg));
                        }
                    }
                }
            }
            command_str
        } else {
            // Use cmd directly as a shell command
            cmd.to_string()
        };

        // Apply variable substitution to the full command
        let processed_command = substitute_variables(&full_command, variables);

        // Execution setup
        let (exec_cmd, args) = if sudo {
            ("sudo", vec!["sh", "-c", &processed_command])
        } else {
            ("sh", vec!["-c", &processed_command])
        };

        if dry_run {
            print_log(
                LogLevel::Info,
                &format!(
                    "Dry-run: Would execute shell command{}: {}",
                    if sudo { " with sudo" } else { "" },
                    processed_command
                ),
            );
        } else {
            if verbose {
                print_log(
                    LogLevel::Info,
                    &format!(
                        "Executing shell command{}: {}",
                        if sudo { " with sudo" } else { "" },
                        processed_command
                    ),
                );
            }

            let output = Command::new(exec_cmd)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;

            if !output.status.success() {
                print_log(
                    LogLevel::Error,
                    &format!(
                        "Shell command failed: {}: {}",
                        processed_command,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                );
            } else if verbose {
                print_log(
                    LogLevel::Success,
                    &format!("Shell command executed: {}", processed_command,),
                );

                if !output.stdout.is_empty() {
                    print_log(
                        LogLevel::CommandOutput,
                        &format!("Output: \n{}", String::from_utf8_lossy(&output.stdout)),
                    );
                }
            }
        }
    }

    Ok(())
}
