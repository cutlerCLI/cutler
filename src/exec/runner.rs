// SPDX-License-Identifier: Apache-2.0

use crate::cli::atomic::should_dry_run;
use crate::config::loader::Config;
use crate::util::logging::{BOLD, LogLevel, RESET, print_log};
use anyhow::{Error, Result, anyhow, bail};
use std::collections::HashMap;
use std::env;
use tokio::process::Command;
use tokio::task;

/// Represents an external command job.
pub struct ExecJob {
    pub name: String,
    pub run: String,
    pub sudo: bool,
    pub ensure_first: bool,
    pub flag: bool,
    pub required: Vec<String>,
}

/// Extract a single command by name from the user config.
pub fn extract_cmd(config: &Config, name: &str) -> Result<ExecJob> {
    let command_map = config
        .command
        .as_ref()
        .ok_or_else(|| anyhow!("no command exists"))?;
    let command = command_map
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow!("no such command {}", name))?;

    // substitute to get possible variables
    // ultimately turning it into the final command to run
    let run = substitute(&command.run, config.vars.as_ref().cloned());

    // extra fields
    let sudo = command.sudo.unwrap_or_default();
    let flag = command.flag.unwrap_or_default();
    let ensure_first = command.ensure_first.unwrap_or_default();
    let required = command.required.clone().unwrap_or_default();

    Ok(ExecJob {
        name: name.to_string(),
        run,
        sudo,
        ensure_first,
        flag,
        required,
    })
}

// Pull all external commands written in user config into state objects.
pub fn extract_all_cmds(config: &Config) -> Vec<ExecJob> {
    let mut jobs = Vec::new();

    if let Some(command_map) = config.command.as_ref() {
        for (name, _) in command_map.iter() {
            if let Ok(job) = extract_cmd(config, name) {
                jobs.push(job);
            }
        }
    }

    jobs
}

/// Perform variable substitution (env + `[external.variables]`) in a text.
fn substitute(text: &str, vars: Option<HashMap<String, String>>) -> String {
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
            .as_ref()
            .and_then(|map| map.get(var_name))
            .cloned()
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
async fn execute_command(job: ExecJob, dry_run: bool) -> Result<()> {
    // build the actual runner
    let (bin, args) = if job.sudo {
        ("sudo", vec!["sh", "-c", &job.run])
    } else {
        ("sh", vec!["-c", &job.run])
    };

    if dry_run {
        print_log(LogLevel::Dry, &format!("Would execute: {bin} {}", job.run));
        return Ok(());
    }

    print_log(LogLevel::Exec, &format!("{BOLD}{}{RESET}", job.name));

    let mut child = Command::new(bin).args(&args).spawn()?;
    let status = child.wait().await?;

    if !status.success() {
        print_log(
            LogLevel::Error,
            &format!("External command failed: {}", job.name),
        );
        return Err(Error::msg("cmd failed"));
    }

    Ok(())
}

/// Helper for: run_all(), run_one()
/// Checks if the binaries designated in `required` are found in $PATH and whether to skip command execution.
fn all_bins_present(required: &[String]) -> bool {
    let mut present = true;

    if required.is_empty() {
        return present;
    }

    for bin in required {
        if which::which(bin).is_err() {
            print_log(LogLevel::Warning, &format!("{bin} not found in $PATH."));
            present = false;
        }
    }

    present
}

/// Execution mode enum.
#[derive(PartialEq)]
pub enum ExecMode {
    Regular,
    All,
    Flagged,
}

/// Run all extracted external commands via `sh -c` (or `sudo sh -c`) in parallel.
/// Returns the amount of successfully executed commmands.
pub async fn run_all(config: Config, mode: ExecMode) -> Result<i32> {
    let cmds = extract_all_cmds(&config);

    // separate ensure_first commands from regular commands
    let mut ensure_first_cmds = Vec::new();
    let mut regular_cmds = Vec::new();

    for job in cmds {
        if !all_bins_present(&job.required)
            || (mode == ExecMode::Regular && job.flag)
            || (mode == ExecMode::Flagged && !job.flag)
        {
            continue;
        } else if job.ensure_first {
            ensure_first_cmds.push(job);
        } else {
            regular_cmds.push(job);
        }
    }

    let dry_run = should_dry_run();

    let mut failures = 0;
    let mut successes = 0;

    // run all ensure_first commands sequentially first
    for job in ensure_first_cmds {
        if (execute_command(job, dry_run).await).is_err() {
            failures += 1;
        } else {
            successes += 1;
        }
    }

    // then run all regular commands concurrently
    let mut handles = Vec::new();
    for job in regular_cmds {
        handles.push(task::spawn(
            async move { execute_command(job, dry_run).await },
        ));
    }

    for handle in handles {
        if handle.await?.is_err() {
            failures += 1;
        } else {
            successes += 1;
        }
    }

    // inspect failures count
    if failures > 0 {
        print_log(
            LogLevel::Warning,
            &format!("{failures} external commands failed"),
        );
    }

    Ok(successes)
}

/// Run exactly one command entry, given its name.
pub async fn run_one(config: Config, name: &str) -> Result<()> {
    let state = extract_cmd(&config, name)?;

    if !all_bins_present(&state.required) {
        bail!("Cannot execute command due to missing binaries.")
    }

    let dry_run = should_dry_run();
    execute_command(state, dry_run).await
}
