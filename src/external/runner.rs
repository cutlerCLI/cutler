use crate::snapshot::state::ExternalCommandState;
use crate::util::logging::{LogLevel, print_log};
use std::env;
use std::process::{Command, Stdio};
use toml::Value;

/// Pull all `[[external.command]]` entries into state objects.
pub fn extract(config: &Value) -> Vec<ExternalCommandState> {
    let mut cmds = Vec::new();
    if let Some(ext) = config.get("external") {
        if let Some(arr) = ext.get("command").and_then(|v| v.as_array()) {
            for cmd_val in arr {
                if let Some(tbl) = cmd_val.as_table() {
                    if let Some(cmd) = tbl.get("cmd").and_then(|v| v.as_str()) {
                        let args = tbl
                            .get("args")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|x| x.as_str())
                                    .map(String::from)
                                    .collect()
                            })
                            .unwrap_or_default();
                        let sudo = tbl.get("sudo").and_then(|v| v.as_bool()).unwrap_or(false);
                        cmds.push(ExternalCommandState {
                            cmd: cmd.into(),
                            args,
                            sudo,
                        });
                    }
                }
            }
        }
    }
    cmds
}

/// Perform variable substitution (env + `[external.variables]`) in a text.
fn substitute(text: &str, vars: Option<&toml::value::Table>) -> String {
    let mut result = text.to_string();
    let mut var_positions = Vec::new();

    // find all variable references
    let mut i = 0;
    while i < result.len() {
        if result[i..].starts_with('$') {
            let start = i;
            i += 1;

            // handle ${var} format
            let is_braced = i < result.len() && result[i..].starts_with('{');
            if is_braced {
                i += 1;
                while i < result.len() && result.chars().nth(i) != Some('}') {
                    i += 1;
                }
                if i < result.len() {
                    i += 1;
                }
            } else {
                // handle $var format - variable name can include alphanumeric and underscore
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

    // process variables from end to start to avoid position shifts
    for (start, end) in var_positions.into_iter().rev() {
        let var_ref = &result[start..end];

        // extract variable name
        let var_name = if var_ref.starts_with("${") && var_ref.ends_with('}') {
            &var_ref[2..var_ref.len() - 1]
        } else {
            &var_ref[1..]
        };

        // try to find value in custom variables
        let replacement = if let Some(vars_map) = vars {
            if let Some(value) = vars_map.get(var_name) {
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

        // if not found in custom variables, try environment
        let final_replacement = match replacement {
            Some(val) => val,
            None => env::var(var_name).unwrap_or_else(|_| var_ref.to_string()),
        };

        // replace in the result string
        result.replace_range(start..end, &final_replacement);
    }

    result
}

/// Run all extracted external commands via `sh -c` (or `sudo sh -c`).
pub fn run_all(config: &Value, verbose: bool, dry_run: bool) -> Result<(), anyhow::Error> {
    let ext = config.get("external").and_then(|v| v.as_table());
    let vars = ext
        .and_then(|t| t.get("variables"))
        .and_then(|v| v.as_table());
    let cmds = extract(config);

    for state in cmds {
        // build a single shell string
        let mut line = state.cmd.clone();
        for arg in &state.args {
            let sub = substitute(arg, vars);
            if sub.contains(' ') {
                line.push_str(&format!(" \"{}\"", sub));
            } else {
                line.push_str(&format!(" {}", sub));
            }
        }
        let final_cmd = substitute(&line, vars);
        let (bin, args) = if state.sudo {
            ("sudo", vec!["sh", "-c", &final_cmd])
        } else {
            ("sh", vec!["-c", &final_cmd])
        };

        if dry_run {
            print_log(
                LogLevel::Info,
                &format!("Dry-run: would exec {} {}", bin, final_cmd),
            );
        } else {
            if verbose {
                print_log(LogLevel::Info, &format!("Exec {} {}", bin, final_cmd));
            }
            let out = Command::new(bin)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;

            if !out.status.success() {
                print_log(
                    LogLevel::Error,
                    &format!(
                        "External command failed: {}: {}",
                        final_cmd,
                        String::from_utf8_lossy(&out.stderr)
                    ),
                );
            } else if verbose && !out.stdout.is_empty() {
                print_log(
                    LogLevel::CommandOutput,
                    &format!("Out: {}", String::from_utf8_lossy(&out.stdout)),
                );
            }
        }
    }
    Ok(())
}
