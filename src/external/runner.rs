use crate::snapshot::state::ExternalCommandState;
use crate::util::logging::{LogLevel, print_log};
use anyhow::{Error, Result};
use rayon::prelude::*;
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
    let mut i = 0;

    // find $… or ${…} spans
    while i < result.len() {
        if result[i..].starts_with('$') {
            let start = i;
            i += 1;

            // ${var}
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
                // $var
                while i < result.len()
                    && result
                        .chars()
                        .nth(i)
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false)
                {
                    i += 1;
                }
            }

            var_positions.push((start, i));
        } else {
            i += 1;
        }
    }

    // replace from back to front
    for (start, end) in var_positions.into_iter().rev() {
        let var_ref = &result[start..end];

        // extract variable name
        let var_name = if var_ref.starts_with("${") && var_ref.ends_with('}') {
            &var_ref[2..var_ref.len() - 1]
        } else {
            &var_ref[1..]
        };

        // first try custom vars
        let replacement = vars
            .and_then(|map| map.get(var_name))
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                other => other.to_string(),
            })
            // else try env
            .or_else(|| env::var(var_name).ok())
            // else keep literal
            .unwrap_or_else(|| var_ref.to_string());

        result.replace_range(start..end, &replacement);
    }

    result
}

/// Run all extracted external commands via `sh -c` (or `sudo sh -c`) in parallel.
pub fn run_all(config: &Value, verbose: bool, dry_run: bool) -> Result<()> {
    let ext = config.get("external").and_then(|v| v.as_table());
    let vars: Option<toml::value::Table> = ext
        .and_then(|t| t.get("variables"))
        .and_then(|v| v.as_table())
        .cloned();
    let cmds = extract(config);

    // run every command in parallel
    let results: Vec<Result<(), Error>> = cmds
        .into_par_iter()
        .map(|state| {
            // cmd str
            let mut line = state.cmd.clone();
            for arg in &state.args {
                let sub = substitute(arg, vars.as_ref());
                if sub.contains(' ') {
                    line.push_str(&format!(" \"{}\"", sub));
                } else {
                    line.push_str(&format!(" {}", sub));
                }
            }
            let final_cmd = substitute(&line, vars.as_ref());

            // choose runner command
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
                return Ok(());
            }

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
                Err(Error::msg("cmd failed"))
            } else {
                if verbose && !out.stdout.is_empty() {
                    print_log(
                        LogLevel::CommandOutput,
                        &format!("Out: {}", String::from_utf8_lossy(&out.stdout)),
                    );
                }
                Ok(())
            }
        })
        .collect();

    // inspect all results here
    let failures = results.into_iter().filter(|r| r.is_err()).count();
    if failures > 0 {
        print_log(
            LogLevel::Warning,
            &format!("{} external commands failed", failures),
        );
    }

    Ok(())
}
