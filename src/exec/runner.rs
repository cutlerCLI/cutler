use crate::snapshot::state::ExternalCommandState;
use crate::util::globals::should_dry_run;
use crate::util::logging::{LogLevel, print_log};
use anyhow::{Error, Result, anyhow};
use std::env;
use std::process::Stdio;
use tokio::process::Command;
use tokio::task;
use toml::Value;

/// Extract a single command by name from the user config.
pub fn extract_cmd(config: &Value, name: &str) -> Result<ExternalCommandState> {
    let vars = config.get("vars").and_then(Value::as_table).cloned();

    let cmd_table = config
        .get("commands")
        .and_then(Value::as_table)
        .and_then(|m| m.get(name))
        .and_then(Value::as_table)
        .ok_or_else(|| anyhow!("no such command '{}'", name))?;

    let template = cmd_table
        .get("run")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("command '{}': missing `run` field", name))?;

    // substitute to get possible variables
    let final_line = substitute(template, vars.as_ref());

    // extra fields
    let sudo = cmd_table
        .get("sudo")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(ExternalCommandState {
        run: final_line,
        sudo,
    })
}

// Pull all external commands written in user config into state objects.
pub fn extract_all_cmds(config: &Value) -> Vec<ExternalCommandState> {
    let mut out = Vec::new();

    if let Some(cmds) = config.get("commands").and_then(Value::as_table) {
        for (name, _) in cmds {
            if let Ok(cmd_state) = extract_cmd(config, name) {
                out.push(cmd_state);
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

/// Helper for: run_one(), run_all()
/// Execute a single command with the given template and sudo flag.
async fn execute_command(
    state: ExternalCommandState,
    vars: Option<&toml::value::Table>,
) -> Result<()> {
    // command execution logic starts here
    let dry_run = should_dry_run();
    let final_cmd = substitute(&state.run, vars);

    // build the actual runner
    let (bin, args) = if state.sudo {
        ("sudo", vec!["sh", "-c", &final_cmd])
    } else {
        ("sh", vec!["-c", &final_cmd])
    };

    if dry_run {
        print_log(LogLevel::Dry, &format!("Would exec {} {}", bin, final_cmd));
        return Ok(());
    }

    print_log(LogLevel::Info, &format!("Exec {} {}", bin, final_cmd));

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
                "External command failed: {}: {}",
                final_cmd,
                String::from_utf8_lossy(&output.stderr)
            ),
        );
        return Err(Error::msg("cmd failed"));
    }

    if !output.stdout.is_empty() {
        print_log(
            LogLevel::CommandOutput,
            &format!("Out: {}", String::from_utf8_lossy(&output.stdout)),
        );
    }
    Ok(())
}

/// Run all extracted external commands via `sh -c` (or `sudo sh -c`) in parallel.
pub async fn run_all(config: &Value) -> Result<()> {
    let vars: Option<toml::value::Table> = config.get("vars").and_then(Value::as_table).cloned();
    let cmds = extract_all_cmds(config);

    // run every command concurrently
    let mut handles = Vec::new();
    for state in cmds {
        let vars = vars.clone();

        handles.push(task::spawn(async move {
            execute_command(state, vars.as_ref()).await
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
pub async fn run_one(config: &Value, which: &str) -> Result<()> {
    let vars = config.get("vars").and_then(Value::as_table).cloned();
    let state = extract_cmd(config, which)?;
    execute_command(state, vars.as_ref()).await
}
