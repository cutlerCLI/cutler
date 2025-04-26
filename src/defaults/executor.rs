use crate::defaults::lock_for;
use crate::util::logging::{LogLevel, print_log};

fn execute_defaults_command(
    command: &str,
    eff_domain: &str,
    eff_key: &str,
    extra_args: Vec<&str>,
    action: &str,
    verbose: bool,
    dry_run: bool,
) -> Result<(), anyhow::Error> {
    let domain_lock = lock_for(eff_domain);
    let _guard = domain_lock.lock().unwrap();

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

pub fn write(
    domain: &str,
    key: &str,
    flag: &str,
    value: &str,
    action: &str,
    verbose: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    execute_defaults_command(
        "write",
        domain,
        key,
        vec![flag, value],
        action,
        verbose,
        dry_run,
    )?;
    Ok(())
}

pub fn delete(
    domain: &str,
    key: &str,
    action: &str,
    verbose: bool,
    dry_run: bool,
) -> Result<(), anyhow::Error> {
    execute_defaults_command("delete", domain, key, vec![], action, verbose, dry_run)?;
    Ok(())
}
