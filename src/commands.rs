use std::fs;
use std::io::{self, Write};
use std::process::Command;

use crate::config::{get_config_path, get_snapshot_path, load_config};
use crate::defaults::{
    execute_defaults_delete, execute_defaults_write, get_flag_and_value, normalize_desired,
};
use crate::domains::{
    check_domain_exists, collect_domains, get_current_value, get_effective_domain,
    get_effective_domain_and_key, needs_prefix,
};
use crate::logging::{LogLevel, print_log};

/// Helper function to prompt user for confirmation
fn confirm_action(prompt: &str) -> io::Result<bool> {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// Applies settings from the configuration file
pub fn apply_defaults(verbose: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        print_log(
            LogLevel::Info,
            &format!("Config file not found at {:?}.", config_path),
        );

        if confirm_action("Would you like to create a new configuration?")? {
            init_config(false, false)?;
            print_log(
                LogLevel::Info,
                "Configuration created. Please review and edit the file before applying.",
            );

            if !confirm_action("Would you like to apply the new configuration now?")? {
                return Ok(());
            }
        } else {
            return Err("No config file present. Exiting.".into());
        }
    }

    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;

    for (domain, settings_table) in &current_domains {
        if needs_prefix(domain) {
            check_domain_exists(&format!("com.apple.{}", domain))?;
        }

        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(domain, key);
            let desired = normalize_desired(value);

            // Skip if value is already set correctly
            if get_current_value(&eff_domain, &eff_key).map_or(false, |curr| curr == desired) {
                if verbose {
                    print_log(
                        LogLevel::Info,
                        &format!("Skipping unchanged setting: {} = {}", eff_key, desired),
                    );
                }
                continue;
            }

            let (flag, value_str) = get_flag_and_value(value)?;
            execute_defaults_write(
                &eff_domain,
                &eff_key,
                flag,
                &value_str,
                "Applying",
                verbose,
                dry_run,
            )?;
        }
    }

    // Copying the snapshot file
    let snapshot_path = get_snapshot_path();
    if dry_run {
        print_log(
            LogLevel::Info,
            &format!(
                "Dry-run: Would copy config file from {:?} to {:?}",
                config_path, snapshot_path
            ),
        );
    } else {
        fs::copy(&config_path, &snapshot_path)?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Snapshot updated at {:?}", snapshot_path),
            );
        }
    }

    if let Err(e) = crate::external::execute_external_commands(&current_parsed, verbose, dry_run) {
        print_log(
            LogLevel::Warning,
            &format!("Failed to execute external commands: {}", e),
        );
    }
    Ok(())
}

/// Unapplies settings using the stored snapshot
pub fn unapply_defaults(verbose: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_path = get_snapshot_path();
    if !snapshot_path.exists() {
        return Err("No snapshot found. Please apply settings first before unapplying.".into());
    }

    let snap_parsed = load_config(&snapshot_path)?;
    let snap_domains = collect_domains(&snap_parsed)?;

    let config_path = get_config_path();
    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;

    if snap_domains != current_domains {
        print_log(
            LogLevel::Warning,
            "Warning: The snapshot (last applied) differs from the current configuration.",
        );

        if !confirm_action("Are you sure you want to unapply everything?")? {
            return Err("Aborted unapply due to configuration differences.".into());
        }
    }

    for (domain, settings_table) in snap_domains {
        if needs_prefix(&domain) {
            check_domain_exists(&format!("com.apple.{}", domain))?;
        }

        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, &key);
            let desired = normalize_desired(&value);

            match get_current_value(&eff_domain, &eff_key) {
                Some(curr) if curr != desired => {
                    print_log(
                        LogLevel::Warning,
                        &format!(
                            "{}.{} has been modified (expected {} but got {}). Skipping removal.",
                            eff_domain, eff_key, desired, curr
                        ),
                    );
                    continue;
                }
                None => continue,
                _ => {}
            }

            execute_defaults_delete(&eff_domain, &eff_key, "Unapplying", verbose, dry_run)?;
        }
    }

    if dry_run {
        print_log(
            LogLevel::Info,
            &format!("Dry-run: Would remove snapshot file at {:?}", snapshot_path),
        );
    } else {
        fs::remove_file(&snapshot_path)?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Snapshot removed from {:?}", snapshot_path),
            );
        }
    }
    Ok(())
}

/// Kills (restarts) Finder, Dock, and SystemUIServer to refresh settings
pub fn restart_system_services(
    verbose: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    const SERVICES: [&str; 3] = ["Finder", "Dock", "SystemUIServer"];

    for &service in &SERVICES {
        if dry_run {
            if verbose {
                print_log(
                    LogLevel::Info,
                    &format!("Dry-run: Would restart {}", service),
                );
            }
        } else {
            let output = Command::new("killall").arg(service).output()?;
            if !output.status.success() {
                print_log(
                    LogLevel::Error,
                    &format!("Failed to restart {}, try restarting manually.", service),
                );
            } else if verbose {
                print_log(LogLevel::Success, &format!("{} restarted.", service));
            }
        }
    }

    if !verbose && !dry_run {
        println!("\n🍎 Done. System services restarted.");
    } else if dry_run {
        println!("\n🍎 Dry-run: System services would be restarted.");
    }
    Ok(())
}

/// Displays the current status comparing the config vs current defaults
pub fn status_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Err(
            "No config file found. Please run 'cutler apply' first, or create a config file."
                .into(),
        );
    }

    let parsed_config = load_config(&config_path)?;
    let domains = collect_domains(&parsed_config)?;
    let mut any_changed = false;

    println!();
    for (domain, settings_table) in domains {
        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, &key);
            let desired = normalize_desired(&value);
            let current =
                get_current_value(&eff_domain, &eff_key).unwrap_or_else(|| "Not set".into());

            let is_changed = current != desired;
            if is_changed {
                any_changed = true;
            }

            if verbose {
                let color = if is_changed {
                    crate::logging::RED
                } else {
                    crate::logging::GREEN
                };
                println!(
                    "{}{} {} -> {} (now {}){}",
                    color,
                    eff_domain,
                    eff_key,
                    desired,
                    current,
                    crate::logging::RESET
                );
            } else if is_changed {
                println!(
                    "{} {} -> should be {} (now {}{}{})",
                    eff_domain,
                    eff_key,
                    desired,
                    crate::logging::RED,
                    current,
                    crate::logging::RESET
                );
            }
        }
    }

    if !any_changed {
        println!("🍎 Nothing to change.");
    } else {
        println!("\nRun `cutler apply` to reapply these changes from your config.")
    }

    Ok(())
}

/// Deletes the configuration file and offers to unapply settings if they are still active
pub fn config_delete(verbose: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if !config_path.exists() {
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("No configuration file found at: {:?}", config_path),
            );
        }
        return Ok(());
    }

    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;
    let mut applied_domains = Vec::new();

    for (domain, _) in current_domains {
        let effective_domain = get_effective_domain(&domain);

        if Command::new("defaults")
            .arg("read")
            .arg(&effective_domain)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            applied_domains.push(effective_domain);
        }
    }

    if !applied_domains.is_empty() {
        println!(
            "The following domains appear to still be applied: {:?}",
            applied_domains
        );
        if confirm_action(
            "Would you like to unapply these settings before deleting the config file?",
        )? {
            unapply_defaults(verbose, dry_run)?;
        }
    }

    let snapshot_path = get_snapshot_path();
    if dry_run {
        print_log(
            LogLevel::Info,
            &format!(
                "Dry-run: Would remove configuration file at {:?}",
                config_path
            ),
        );
        if snapshot_path.exists() {
            print_log(
                LogLevel::Info,
                &format!("Dry-run: Would remove snapshot file at {:?}", snapshot_path),
            );
        }
    } else {
        fs::remove_file(&config_path)?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Configuration file deleted from: {:?}", config_path),
            );
        }
        if snapshot_path.exists() {
            fs::remove_file(&snapshot_path)?;
        }
    }
    Ok(())
}

/// Displays the contents of the configuration file to the terminal.
pub fn config_show(verbose: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Err("Configuration file does not exist.".into());
    }

    if dry_run {
        print_log(
            LogLevel::Info,
            &format!("Dry-run: Would display config file at {:?}", config_path),
        );
        return Ok(());
    }

    let content = fs::read_to_string(&config_path)?;
    println!("{}", content);
    if verbose {
        print_log(LogLevel::Info, "Displayed configuration file.");
    }
    Ok(())
}

/// Initializes a new cutler configuration file with sensible defaults.
pub fn init_config(verbose: bool, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    // Check if config already exists
    if config_path.exists() && !force {
        print_log(
            LogLevel::Warning,
            &format!("Configuration file already exists at {:?}", config_path),
        );
        if !confirm_action("Do you want to overwrite it?")? {
            return Err("Configuration initialization aborted.".into());
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create default configuration based on the advanced example
    let default_config = r#"# Generated with cutler
# See https://github.com/hitblast/cutler for more examples and documentation

[menuextra.clock]
FlashDateSeparators = true
DateFormat = "\"HH:mm:ss\""
Show24Hour = true
ShowAMPM = false
ShowDate = 2
ShowDayOfWeek = false
ShowSeconds = true

[finder]
AppleShowAllFiles = true
CreateDesktop = false
ShowPathbar = true
FXRemoveOldTrashItems = true

[AppleMultitouchTrackpad]
FirstClickThreshold = 0
TrackpadThreeFingerDrag = true

[dock]
tilesize = 50
autohide = true
magnification = false
orientation = "right"
mineffect = "suck"
autohide-delay = 0
autohide-time-modifier = 0.6
expose-group-apps = true

[NSGlobalDomain.com.apple.keyboard]
fnState = false

# Examples of external commands (commented out by default)
# Uncomment and modify as needed

# [external.variables]
# hostname = "my-macbook"

# [external]
# [[external.command]]
# cmd = "scutil"
# args = ["--set", "ComputerName", "$hostname"]
# sudo = true

# [[external.command]]
# cmd = "scutil"
# args = ["--set", "HostName", "$hostname"]
# sudo = true

# [[external.command]]
# cmd = "scutil"
# args = ["--set", "LocalHostName", "$hostname"]
# sudo = true
"#;

    // Write the configuration file
    fs::write(&config_path, default_config).map_err(|e| {
        format!(
            "Failed to write configuration file at {:?}: {}",
            config_path, e
        )
    })?;

    if verbose {
        print_log(
            LogLevel::Success,
            &format!("Configuration file created at: {:?}", config_path),
        );
        print_log(
            LogLevel::Info,
            "Review and edit this file to customize your Mac settings",
        );
    } else {
        println!("🍎 New configuration created at {:?}", config_path);
        println!("Review and customize this file, then run `cutler apply` to apply settings");
    }

    Ok(())
}
