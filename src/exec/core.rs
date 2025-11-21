// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::atomic::should_dry_run;
use crate::config::core::Config;
use crate::util::logging::{BOLD, RESET};
use crate::{log_dry, log_exec, log_warn};
use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
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
        .ok_or_else(|| anyhow!("no such command {name}"))?;

    // substitute to get possible variables
    // ultimately turning it into the final command to run
    let run = substitute(&command.run, config.vars.clone())?;

    // extra fields
    let sudo = command.sudo.unwrap_or_default();
    let flag = command.flag.unwrap_or_default();
    let ensure_first = command.ensure_first.unwrap_or_default();
    let required = command.required.unwrap_or_default();

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
#[must_use]
pub fn extract_all_cmds(config: &Config) -> Vec<ExecJob> {
    let mut jobs = Vec::new();

    if let Some(command_map) = config.command.as_ref() {
        for name in command_map.keys() {
            if let Ok(job) = extract_cmd(config, name) {
                jobs.push(job);
            }
        }
    }

    jobs
}

/// Perform variable substitution (env + `[external.variables]`) in a text.
/// Uses regex to find $var and ${var} patterns.
fn substitute(text: &str, vars: Option<HashMap<String, String>>) -> Result<String> {
    // regex to match $var or ${var}
    // $VAR_NAME or ${VAR_NAME}
    // note: $ followed by [A-Za-z_][A-Za-z0-9_]* or ${...}
    let re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)|\$\{([A-Za-z_][A-Za-z0-9_]*)\}")
        .with_context(
            || "Failed to construct regex pattern for external cmd variable substitution.",
        )?;

    // clusure to resolve variable name
    let resolve_var = |var_name: &str| {
        vars.as_ref()
            .and_then(|map| map.get(var_name))
            .cloned()
            .or_else(|| env::var(var_name).ok())
            .unwrap_or_else(|| format!("${{{var_name}}}"))
    };

    // replace all matches
    let result = re.replace_all(text, |caps: &regex::Captures| {
        // caps[1] is for $var, caps[2] is for ${var}
        let var_name = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map_or("", |m| m.as_str());
        resolve_var(var_name)
    });

    Ok(result.into_owned())
}

/// Helper for: `run_one()`, `run_all()`
/// Execute a single command with the given template and sudo flag.
async fn execute_command(job: ExecJob, dry_run: bool) -> Result<()> {
    // build the actual runner
    let (bin, args) = if job.sudo {
        ("sudo", vec!["sh", "-c", &job.run])
    } else {
        ("sh", vec!["-c", &job.run])
    };

    if dry_run {
        log_dry!("Would execute: {bin} {}", job.run);
        return Ok(());
    }

    log_exec!("{BOLD}{}{RESET}", job.name);

    let mut child = Command::new(bin).args(&args).spawn()?;
    let status = child.wait().await?;

    if !status.success() {
        bail!(format!("Command {} failed to execute.", job.name))
    }

    Ok(())
}

/// Helper for: `run_all()`, `run_one()`
/// Checks if the binaries designated in `required` are found in $PATH and whether to skip command execution.
fn all_bins_present(required: &[String]) -> bool {
    let mut present = true;

    if !required.is_empty() {
        for bin in required {
            if which::which(bin).is_err() {
                log_warn!("{bin} not found in $PATH.");
                present = false;
            }
        }
    }

    present
}

/// Execution mode enum.
#[derive(PartialEq, Eq)]
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

    // inspect count
    if failures > 0 {
        log_warn!("{failures} external commands failed",);
    } else if successes == 0 {
        log_warn!("No regular external commands found. Maybe you meant flagged or all?",);
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
