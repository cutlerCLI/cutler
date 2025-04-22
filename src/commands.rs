use semver::Version;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
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
use crate::logging::{BOLD, LogLevel, RESET, print_log};
use crate::snapshot::{SettingState, Snapshot, get_snapshot_path};

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
            return Ok(());
        } else {
            return Err("No config file present. Exiting.".into());
        }
    }

    let current_parsed = load_config(&config_path)?;
    let current_domains = collect_domains(&current_parsed)?;

    // Load existing snapshot if it exists
    let snapshot_path = get_snapshot_path();
    let mut snapshot = if snapshot_path.exists() {
        match Snapshot::load_from_file(&snapshot_path) {
            Ok(snap) => snap,
            Err(e) => {
                print_log(
                    LogLevel::Warning,
                    &format!(
                        "Could not load existing snapshot: {}. Creating a new one.",
                        e
                    ),
                );
                Snapshot::new()
            }
        }
    } else {
        Snapshot::new()
    };

    // Create a HashMap for quicker lookups of existing settings in the snapshot
    let mut existing_settings = HashMap::new();
    for setting in &snapshot.settings {
        existing_settings.insert(
            (setting.domain.clone(), setting.key.clone()),
            setting.clone(),
        );
    }

    // Track which settings we've seen in this run (to identify removals)
    let mut seen_settings = HashSet::new();

    // Process each setting from the config
    for (domain, settings_table) in &current_domains {
        if needs_prefix(domain) {
            check_domain_exists(&format!("com.apple.{}", domain))?;
        }

        for (key, value) in settings_table {
            let (eff_domain, eff_key) = get_effective_domain_and_key(domain, key);
            let desired = normalize_desired(value);

            seen_settings.insert((eff_domain.clone(), eff_key.clone()));

            // Get current system value
            let current_value = get_current_value(&eff_domain, &eff_key);

            // Check if we already have this setting in our snapshot
            let key_pair = (eff_domain.clone(), eff_key.clone());
            let existing_entry = existing_settings.get(&key_pair);

            if let Some(existing) = existing_entry {
                // Setting already in snapshot - check if config value changed
                if existing.new_value != desired {
                    if verbose {
                        print_log(
                            LogLevel::Info,
                            &format!(
                                "Value changed in config for {}.{}: {} -> {}",
                                eff_domain, eff_key, existing.new_value, desired
                            ),
                        );
                    }

                    let (flag, value_str) = get_flag_and_value(value)?;
                    execute_defaults_write(
                        &eff_domain,
                        &eff_key,
                        flag,
                        &value_str,
                        "Updating",
                        verbose,
                        dry_run,
                    )?;

                    // Update snapshot with new desired value
                    // Keep original_value from first application
                    existing_settings.insert(
                        key_pair,
                        SettingState {
                            domain: eff_domain.clone(),
                            key: eff_key.clone(),
                            original_value: existing.original_value.clone(),
                            new_value: desired,
                        },
                    );
                } else if verbose {
                    print_log(
                        LogLevel::Info,
                        &format!("Skipping unchanged setting: {}.{}", eff_domain, eff_key),
                    );
                }
            } else {
                // New setting not yet in snapshot
                if current_value.as_ref() == Some(&desired) {
                    if verbose {
                        print_log(
                            LogLevel::Info,
                            &format!("Value already set correctly for {}.{}", eff_domain, eff_key),
                        );
                    }

                    // Add to snapshot with current as both original and new
                    existing_settings.insert(
                        key_pair,
                        SettingState {
                            domain: eff_domain.clone(),
                            key: eff_key.clone(),
                            original_value: current_value.clone(),
                            new_value: desired.clone(),
                        },
                    );
                } else {
                    // Value needs to be set
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

                    // Add to snapshot
                    existing_settings.insert(
                        key_pair,
                        SettingState {
                            domain: eff_domain.clone(),
                            key: eff_key.clone(),
                            original_value: current_value,
                            new_value: desired,
                        },
                    );
                }
            }
        }
    }

    // Rebuild the settings list from our updated HashMap
    snapshot.settings = existing_settings.into_values().collect();

    // External commands
    if dry_run {
        print_log(
            LogLevel::Info,
            &format!("Dry-run: Would save snapshot to {:?}", snapshot_path),
        );
    } else {
        // Store external commands in snapshot
        // This helps cutler to skip external commands if applied multiple times
        snapshot.external_commands = crate::external::extract_external_commands(&current_parsed);

        // Save the snapshot file first, before executing external commands
        snapshot.save_to_file(&snapshot_path)?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!(
                    "Snapshot saved to {:?} with {} settings",
                    snapshot_path,
                    snapshot.settings.len()
                ),
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

/// Executes only external commands from the configuration file
pub fn execute_only_external_commands(
    verbose: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
            return Ok(());
        } else {
            return Err("No config file present. Exiting.".into());
        }
    }

    let current_parsed = load_config(&config_path)?;

    // Load existing snapshot if it exists
    let snapshot_path = get_snapshot_path();
    let mut snapshot = if snapshot_path.exists() {
        match Snapshot::load_from_file(&snapshot_path) {
            Ok(snap) => snap,
            Err(e) => {
                print_log(
                    LogLevel::Warning,
                    &format!(
                        "Could not load existing snapshot: {}. Creating a new one.",
                        e
                    ),
                );
                Snapshot::new()
            }
        }
    } else {
        Snapshot::new()
    };

    print_log(
        LogLevel::Info,
        "Executing only external commands from config (skipping defaults)",
    );

    // Store external commands in snapshot
    snapshot.external_commands = crate::external::extract_external_commands(&current_parsed);

    // Save the snapshot file before executing external commands
    if dry_run {
        print_log(
            LogLevel::Info,
            &format!(
                "Dry-run: Would update snapshot at {:?} with external commands",
                snapshot_path
            ),
        );
    } else {
        snapshot.save_to_file(&snapshot_path)?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!(
                    "Snapshot updated at {:?} with external commands",
                    snapshot_path
                ),
            );
        }
    }

    // Execute only external commands
    if let Err(e) = crate::external::execute_external_commands(&current_parsed, verbose, dry_run) {
        print_log(
            LogLevel::Warning,
            &format!("Failed to execute external commands: {}", e),
        );
        return Err(e);
    }

    if !verbose && !dry_run {
        println!("\n🍎 External commands executed successfully.");
    } else if dry_run {
        println!("\n🍎 Dry-run: External commands would have been executed.");
    }

    Ok(())
}

/// Unapplies settings using the stored snapshot
pub fn unapply_defaults(verbose: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_path = get_snapshot_path();
    if !snapshot_path.exists() {
        return Err("No snapshot found. Please apply settings first before unapplying.\nAs a fallback, you can use 'cutler reset' to reset settings to defaults.".into());
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
            "External commands were executed previously. Make sure to revert them manually.",
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

/// Displays the contents of the configuration file to the terminal
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

/// Initializes a new cutler configuration file with sensible defaults
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

/// Checks for updates to cutler by fetching the latest version from GitHub's Cargo.toml
pub fn check_for_updates(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = env!("CARGO_PKG_VERSION");

    if verbose {
        print_log(
            LogLevel::Info,
            &format!("Current version: {}", current_version),
        );
        print_log(LogLevel::Info, "Checking for updates...");
    } else {
        println!("Checking for updates...");
    }

    // URL for the Cargo.toml file in the GitHub repository
    let cargo_url = "https://raw.githubusercontent.com/hitblast/cutler/main/Cargo.toml";
    let response = ureq::get(cargo_url).call();

    let response = match response {
        Ok(r) => r,
        Err(e) => return Err(format!("Failed to check for updates: {}", e).into()),
    };

    let cargo_toml: String = response.into_body().read_to_string()?;
    let parsed: toml::Value = cargo_toml.parse()?;

    // Extract the latest version from the Cargo.toml [package] section
    let latest_version = parsed
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or("Invalid or missing version in remote Cargo.toml")?;

    if verbose {
        print_log(
            LogLevel::Info,
            &format!("Latest version: {}", latest_version),
        );
    }

    // Parse versions for comparison
    let current = Version::parse(current_version)?;
    let latest = Version::parse(latest_version)?;

    // Compare versions
    match current.cmp(&latest) {
        Ordering::Less => {
            println!(
                "\n{}Update available:{} {} → {}",
                BOLD, RESET, current_version, latest_version
            );

            // Show update instructions
            println!("\nTo update, run one of the following:");
            println!("  brew upgrade hitblast/tap/cutler    # if installed with Homebrew");
            println!("  cargo install cutler --force        # if installed with Cargo");
            println!("\nOr download the latest release from:");
            println!("  https://github.com/hitblast/cutler/releases");
        }
        Ordering::Equal => {
            print_log(LogLevel::Success, "You are using the latest version.");
        }
        Ordering::Greater => {
            print_log(
                LogLevel::Info,
                &format!(
                    "You are using a development version ({}) ahead of the latest release ({}).",
                    current_version, latest_version
                ),
            );
        }
    }

    Ok(())
}
