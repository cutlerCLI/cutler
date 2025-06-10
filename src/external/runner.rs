use crate::snapshot::state::ExternalCommandState;
use crate::util::logging::{LogLevel, print_log};
use anyhow::{Error, Result, anyhow};
use std::env;
use std::process::Stdio;
use tokio::process::Command;
use tokio::task;
use toml::Value;

// Pull all commands into state objects.
pub fn extract(config: &Value) -> Vec<ExternalCommandState> {
    let vars = config.get("vars").and_then(Value::as_table).cloned();
    let mut out = Vec::new();

    if let Some(cmds) = config.get("commands").and_then(Value::as_table) {
        for (_, tbl) in cmds {
            if let Value::Table(tbl) = tbl {
                // each command must have a "run = ..." block
                if let Some(template) = tbl.get("run").and_then(Value::as_str) {
                    // substitute to get possible varriables
                    let final_line = substitute(template, vars.as_ref());
                    let sudo = tbl.get("sudo").and_then(Value::as_bool).unwrap_or(false);

                    out.push(ExternalCommandState {
                        run: final_line,
                        sudo,
                    })
                }
            }
        }
    }

    out
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

    // replace it from back to front
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
pub async fn run_all(config: &Value, verbose: bool, dry_run: bool) -> Result<()> {
    let vars: Option<toml::value::Table> = config.get("vars").and_then(Value::as_table).cloned();
    let cmds = extract(config);

    // run every command concurrently
    let mut handles = Vec::new();
    for state in cmds {
        let vars = vars.clone();
        handles.push(task::spawn(async move {
            let final_cmd = substitute(&state.run, vars.as_ref());
            let (bin, args) = if state.sudo {
                ("sudo", vec!["sh", "-c", &final_cmd])
            } else {
                ("sh", vec!["-c", &final_cmd])
            };
            if dry_run {
                print_log(LogLevel::Dry, &format!("Would exec {} {}", bin, final_cmd));
                return Ok::<(), Error>(());
            }
            if verbose {
                print_log(LogLevel::Info, &format!("Exec {} {}", bin, final_cmd));
            }
            let child = Command::new(bin)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let out = child.wait_with_output().await?;
            if !out.status.success() {
                print_log(
                    LogLevel::Error,
                    &format!(
                        "External command failed: {}: {}",
                        final_cmd,
                        String::from_utf8_lossy(&out.stderr)
                    ),
                );
                return Err(Error::msg("cmd failed"));
            }
            if verbose && !out.stdout.is_empty() {
                print_log(
                    LogLevel::CommandOutput,
                    &format!("Out: {}", String::from_utf8_lossy(&out.stdout)),
                );
            }
            Ok::<(), Error>(())
        }));
    }
    let mut failures = 0;
    for handle in handles {
        if handle.await.unwrap().is_err() {
            failures += 1;
        }
    }

    // inspect failures count
    if failures > 0 {
        print_log(
            LogLevel::Warning,
            &format!("{} external commands failed", failures),
        );
    }

    Ok(())
}

/// Run exactly one command entry, given its name.
pub async fn run_one(config: &Value, which: &str, verbose: bool, dry_run: bool) -> Result<()> {
    let vars = config.get("vars").and_then(Value::as_table).cloned();

    let cmd_table = config
        .get("commands")
        .and_then(Value::as_table)
        .and_then(|m| m.get(which))
        .and_then(Value::as_table)
        .ok_or_else(|| anyhow!("no such command '{}'", which))?;

    let template = cmd_table
        .get("run")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("command '{}': missing `run` field", which))?;

    let final_cmd = substitute(template, vars.as_ref());

    let sudo = cmd_table
        .get("sudo")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    // build the actual runner
    let (bin, args) = if sudo {
        ("sudo", vec!["sh", "-c", &final_cmd])
    } else {
        ("sh", vec!["-c", &final_cmd])
    };

    if dry_run {
        print_log(LogLevel::Dry, &format!("Would exec {} {}", bin, final_cmd));
        return Ok(());
    }
    if verbose {
        print_log(LogLevel::Info, &format!("Exec {} {}", bin, final_cmd));
    }

    let child = Command::new(bin)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    if !output.status.success() {
        print_log(
            LogLevel::Error,
            &format!(
                "External command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        );
        Err(Error::msg("cmd failed"))
    } else {
        if verbose && !output.stdout.is_empty() {
            print_log(
                LogLevel::CommandOutput,
                &format!("Out: {}", String::from_utf8_lossy(&output.stdout)),
            );
        }
        Ok(())
    }
}
