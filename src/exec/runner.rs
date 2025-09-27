// SPDX-License-Identifier: Apache-2.0

use crate::cli::atomic::should_dry_run;
use crate::snapshot::state::ExternalCommandState;
use crate::util::logging::{LogLevel, print_log};
use anyhow::{Error, Result, anyhow, bail};
use std::{env, process::Stdio};
use tokio::process::Command;
use tokio::task;
use toml::{Table, Value};

/// Extract a single command by name from the user config.
pub fn extract_cmd(config: &Table, name: &str) -> Result<ExternalCommandState> {
    let vars = config.get("vars").and_then(Value::as_table).cloned();

    let cmd_table = config
        .get("command")
        .or_else(|| config.get("commands"))
        .and_then(Value::as_table)
        .and_then(|m| m.get(name))
        .and_then(Value::as_table)
        .ok_or_else(|| anyhow!("No such command '{}'", name))?;

    let template = cmd_table
        .get("run")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Command '{}': missing `run` field", name))?;

    // substitute to get possible variables
    // ultimately turning it into the final command to run
    let run = substitute(template, vars.as_ref());

    // extra fields
    let sudo = cmd_table
        .get("sudo")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let ensure_first = cmd_table
        .get("ensure_first")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let required: Vec<String> = cmd_table
        .get("required")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(ExternalCommandState {
        name: name.to_string(),
        run,
        sudo,
        ensure_first,
        required,
    })
}

// Pull all external commands written in user config into state objects.
pub fn extract_all_cmds(config: &Table) -> Vec<ExternalCommandState> {
    if let Some(cmds) = config
        .get("command")
        .or_else(|| config.get("commands"))
        .and_then(Value::as_table)
    {
        let output: Vec<ExternalCommandState> = cmds
            .iter()
            .filter_map(|(name, _)| extract_cmd(config, name).ok())
            .collect();

        return output;
    }

    Vec::new()
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

/// Helper for: run_one(), run_all()
/// Execute a single command with the given template and sudo flag.
async fn execute_command(
    state: ExternalCommandState,
    vars: Option<&toml::value::Table>,
    dry_run: bool,
) -> Result<()> {
    // command execution logic starts here
    let final_cmd = substitute(&state.run, vars).trim().to_string();

    // build the actual runner
    let (bin, args) = if state.sudo {
        ("sudo", vec!["sh", "-c", &final_cmd])
    } else {
        ("sh", vec!["-c", &final_cmd])
    };

    if dry_run {
        print_log(LogLevel::Dry, &format!("Would execute: {bin} {final_cmd}"));
        return Ok(());
    }

    print_log(LogLevel::Exec, &format!("Execute: {bin} {final_cmd}"));

    // Inherit stdin, stdout, and stderr so the user can interact with the command
    let mut child = Command::new(bin)
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let status = child.wait().await?;
    if !status.success() {
        print_log(
            LogLevel::Error,
            &format!("External command failed: {}", state.name),
        );
        return Err(Error::msg("cmd failed"));
    }

    Ok(())
}

/// Helper for: run_all(), run_one()
/// Checks if the binaries designated in `required` are found in $PATH and whether to skip command execution.
fn should_skip_exec(required: &[String]) -> bool {
    let mut skip_exec = false;

    if required.is_empty() {
        return skip_exec;
    }

    for bin in required {
        if which::which(bin).is_err() {
            print_log(LogLevel::Warning, &format!("{bin} not found in $PATH."));
            skip_exec = true;
        }
    }

    skip_exec
}

/// Run all extracted external commands via `sh -c` (or `sudo sh -c`) in parallel.
pub async fn run_all(config: &Table) -> Result<()> {
    print_log(
        LogLevel::Warning,
        "If you are using the [commands] table, switch to [command] as it will be deprecated soon.",
    );
    let cmds = extract_all_cmds(config);

    // separate ensure_first commands from regular commands
    let mut ensure_first_cmds = Vec::new();
    let mut regular_cmds = Vec::new();

    for state in cmds {
        if should_skip_exec(&state.required) {
            continue;
        } else if state.ensure_first {
            ensure_first_cmds.push(state);
        } else {
            regular_cmds.push(state);
        }
    }

    let dry_run = should_dry_run();
    let vars: Option<toml::value::Table> = config.get("vars").and_then(Value::as_table).cloned();

    let mut failures = 0;

    // run all ensure_first commands sequentially first
    for state in ensure_first_cmds {
        if (execute_command(state, vars.as_ref(), dry_run).await).is_err() {
            failures += 1;
        }
    }

    // then run all regular commands concurrently
    let mut handles = Vec::new();
    for state in regular_cmds {
        let vars = vars.clone();
        handles.push(task::spawn(async move {
            execute_command(state, vars.as_ref(), dry_run).await
        }));
    }

    for handle in handles {
        if handle.await.unwrap().is_err() {
            failures += 1;
        }
    }

    // inspect failures count
    if failures > 0 {
        print_log(
            LogLevel::Warning,
            &format!("{failures} external commands failed"),
        );
    }

    Ok(())
}

/// Run exactly one command entry, given its name.
pub async fn run_one(config: &Table, which: &str) -> Result<()> {
    let state = extract_cmd(config, which)?;

    print_log(
        LogLevel::Warning,
        "If you are using the [commands] table, switch to [command] as it will be deprecated soon.",
    );

    if should_skip_exec(&state.required) {
        bail!("Cannot execute command due to missing binaries.")
    }

    let dry_run = should_dry_run();
    let vars = config.get("vars").and_then(Value::as_table).cloned();
    execute_command(state, vars.as_ref(), dry_run).await
}
