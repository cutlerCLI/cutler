use std::fs;
use std::io::{self, Write};
use std::process::Command;

use crate::config::{get_config_path, load_config};
use crate::defaults::{
    execute_defaults_delete, execute_defaults_write, get_flag_and_value, get_flag_for_value,
    normalize_desired,
};
use crate::domains::{
    check_domain_exists, collect_domains, get_current_value, get_effective_domain,
    get_effective_domain_and_key, needs_prefix,
};
use crate::logging::{LogLevel, print_log};
use crate::snapshot::{ExternalCommandState, SettingState, Snapshot, get_snapshot_path};

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

    // Create a new snapshot
    let mut snapshot = Snapshot::new();

    for (domain, settings_table) in &current_domains {
        if needs_prefix(domain) {
            check_domain_exists(&format!("com.apple.{}", domain))?;
        }

        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(domain, key);
            let desired = normalize_desired(value);

            // Capture the original value before changing it
            let original_value = get_current_value(&eff_domain, &eff_key);

            // Skip if value is already set correctly
            if original_value.as_ref() == Some(&desired) {
                if verbose {
                    print_log(
                        LogLevel::Info,
                        &format!("Skipping unchanged setting: {} = {}", eff_key, desired),
                    );
                }
                continue;
            }

            // Store in snapshot
            snapshot.settings.push(SettingState {
                domain: eff_domain.clone(),
                key: eff_key.clone(),
                original_value,
                new_value: desired.clone(),
            });

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

    // Save the snapshot file first, before executing external commands
    let snapshot_path = get_snapshot_path();
    if dry_run {
        print_log(
            LogLevel::Info,
            &format!("Dry-run: Would save snapshot to {:?}", snapshot_path),
        );
    } else {
        // Store external commands in snapshot
        // This helps cutler to skip external commands if applied multiple times
        if let Some(ext_section) = current_parsed.get("external") {
            if let Some(commands_array) = ext_section.get("command").and_then(|v| v.as_array()) {
                for command_val in commands_array {
                    if let Some(command_table) = command_val.as_table() {
                        if let Some(cmd) = command_table.get("cmd").and_then(|v| v.as_str()) {
                            let args: Vec<String> = if let Some(arg_val) = command_table.get("args")
                            {
                                if let Some(arr) = arg_val.as_array() {
                                    arr.iter()
                                        .filter_map(|a| a.as_str())
                                        .map(|s| s.to_string())
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

                            snapshot.external_commands.push(ExternalCommandState {
                                cmd: cmd.to_string(),
                                args,
                                sudo,
                            });
                        }
                    }
                }
            }
        }

        snapshot.save_to_file(&snapshot_path)?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Snapshot saved to {:?}", snapshot_path),
            );
        }
    }

    // Execute external commands using existing function
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

    // Load the snapshot
    let snapshot = Snapshot::load_from_file(&snapshot_path)?;

    // Unapply settings in reverse order (to handle dependencies correctly)
    for setting in snapshot.settings.iter().rev() {
        match &setting.original_value {
            Some(orig_val) => {
                // Restore to original value
                let (flag, value_str) = get_flag_for_value(orig_val)?;

                execute_defaults_write(
                    &setting.domain,
                    &setting.key,
                    flag,
                    &value_str,
                    "Restoring",
                    verbose,
                    dry_run,
                )?;

                if verbose {
                    print_log(
                        LogLevel::Success,
                        &format!(
                            "Restored {}.{} to original value: {}",
                            setting.domain, setting.key, orig_val
                        ),
                    );
                }
            }
            None => {
                // Setting didn't exist before, so delete it
                execute_defaults_delete(
                    &setting.domain,
                    &setting.key,
                    "Removing",
                    verbose,
                    dry_run,
                )?;

                if verbose {
                    print_log(
                        LogLevel::Success,
                        &format!(
                            "Removed {}.{} (didn't exist before cutler)",
                            setting.domain, setting.key
                        ),
                    );
                }
            }
        }
    }

    // Note about external commands not being reverted
    if !snapshot.external_commands.is_empty() {
        print_log(
            LogLevel::Warning,
            &format!(
                "{} external commands were executed but cannot be automatically reverted.",
                snapshot.external_commands.len()
            ),
        );
    }

    // Remove the snapshot file
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
            "No config file found. Please run 'cutler init' first, or create a config file.".into(),
        );
    }

    let parsed_config = load_config(&config_path)?;
    let domains = collect_domains(&parsed_config)?;

    println!("\n{} Current Status:", crate::logging::BOLD);

    let mut any_different = false;

    for (domain, settings_table) in domains {
        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(&domain, &key);
            let desired = normalize_desired(&value);
            let current =
                get_current_value(&eff_domain, &eff_key).unwrap_or_else(|| "Not set".into());

            let is_different = current != desired;
            if is_different {
                any_different = true;
                println!(
                    "{}{}.{}: should be {} (currently {}{}{}){}",
                    crate::logging::BOLD,
                    eff_domain,
                    eff_key,
                    desired,
                    crate::logging::RED,
                    current,
                    crate::logging::RESET,
                    crate::logging::RESET
                );
            } else if verbose {
                println!(
                    "{}{}.{}: {} (matches desired value){}",
                    crate::logging::GREEN,
                    eff_domain,
                    eff_key,
                    current,
                    crate::logging::RESET
                );
            }
        }
    }

    if !any_different {
        println!("🍎 All settings already match your configuration.");
    } else {
        println!("\nRun `cutler apply` to apply these changes from your config.");
    }

    Ok(())
}

/// Hard resets all domains from the config file without using the snapshot
pub fn reset_defaults(
    verbose: bool,
    dry_run: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Err(
            "No config file found. Please run 'cutler init' first, or create a config file.".into(),
        );
    }

    print_log(
        LogLevel::Warning,
        "This command will DELETE all settings defined in your config file.",
    );
    print_log(
        LogLevel::Warning,
        "Settings will be reset to macOS defaults, not to their previous values.",
    );

    if !force && !confirm_action("Are you sure you want to continue?")? {
        return Ok(());
    }

    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;

    for (domain, settings_table) in &current_domains {
        for (key, _) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(domain, key);

            // Check if the key exists before trying to delete it
            let exists = get_current_value(&eff_domain, &eff_key).is_some();

            if exists {
                execute_defaults_delete(&eff_domain, &eff_key, "Resetting", verbose, dry_run)?;

                if verbose {
                    print_log(
                        LogLevel::Success,
                        &format!("Reset {}.{} to system default", eff_domain, eff_key),
                    );
                }
            } else if verbose {
                print_log(
                    LogLevel::Info,
                    &format!("Skipping {}.{} (not set)", eff_domain, eff_key),
                );
            }
        }
    }

    // Also remove the snapshot file if it exists
    let snapshot_path = get_snapshot_path();
    if snapshot_path.exists() {
        if dry_run {
            print_log(
                LogLevel::Info,
                &format!("Dry-run: Would remove snapshot file at {:?}", snapshot_path),
            );
        } else if let Err(e) = fs::remove_file(&snapshot_path) {
            print_log(
                LogLevel::Warning,
                &format!("Failed to remove snapshot file: {}", e),
            );
        } else if verbose {
            print_log(
                LogLevel::Success,
                &format!("Removed snapshot file at {:?}", snapshot_path),
            );
        }
    }

    println!("\n🍎 Reset complete. All configured settings have been removed.");

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
