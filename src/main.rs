// Standard library imports.
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

// Third-party imports.
use clap::{Parser, Subcommand};
use toml::Value;

// ANSI color escape codes.
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

/// Fast macOS defaults manager for your terminal.
#[derive(Parser)]
#[command(name = "cutler", version, about)]
struct Cli {
    /// Increase output verbosity
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply defaults from the config file.
    Apply,
    /// Unapply (delete) defaults from the config file.
    Unapply,
    /// Delete the configuration file.
    Delete,
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Apply => apply_defaults(cli.verbose),
        Commands::Unapply => unapply_defaults(cli.verbose),
        Commands::Delete => delete_config(cli.verbose),
    };

    match result {
        Ok(_) => {
            if cli.verbose {
                println!(
                    "{}[SUCCESS] Process completed successfully.{}",
                    GREEN, RESET
                );
            } else {
                println!("🍺 Done!");
            }
            match cli.command {
                Commands::Apply | Commands::Unapply => {
                    if let Err(e) = restart_system_services(cli.verbose) {
                        eprintln!("{}[ERROR] Failed to restart services: {}{}", RED, e, RESET);
                    }
                }
                Commands::Delete => {}
            }
        }
        Err(e) => {
            eprintln!("{}[ERROR] {}{}", RED, e, RESET);
            std::process::exit(1);
        }
    }
}

/// Returns the path to the config file, respecting XDG_CONFIG_HOME if available.
fn get_config_path() -> PathBuf {
    if let Some(xdg_config) = env::var_os("XDG_CONFIG_HOME") {
        let mut config_path = PathBuf::from(xdg_config);
        config_path.push("cutler");
        config_path.push("config.toml");
        config_path
    } else if let Some(home) = env::var_os("HOME") {
        let mut config_path = PathBuf::from(home);
        config_path.push(".config");
        config_path.push("cutler");
        config_path.push("config.toml");
        config_path
    } else {
        // Fallback to a relative path if HOME is not set.
        PathBuf::from("config.toml")
    }
}

/// When no config file is present, create an example one.
fn create_example_config(path: &PathBuf, verbose: bool) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let example = r#"
# This is just a basic example of the configuration file.
# Learn more: https://github.com/hitblast/cutler

[dock]
autohide = true
    "#;
    fs::write(path, example.trim_start())?;
    if verbose {
        println!(
            "{}[SUCCESS] Example config created at: {:?}{}",
            GREEN, path, RESET
        );
    } else {
        println!("📝 Example config written to {:?}", path);
    }
    Ok(())
}

/// Recursively flattens a TOML table into a vector of (domain, settings_table) pairs.
/// For example, [menuextra.clock] becomes a domain string "menuextra.clock".
fn flatten_domains(
    prefix: Option<String>,
    table: &toml::value::Table,
    dest: &mut Vec<(String, toml::value::Table)>,
) {
    // Temporary table to collect non-table keys.
    let mut flat_table = toml::value::Table::new();
    let mut nested_tables = toml::value::Table::new();

    for (key, value) in table {
        match value {
            Value::Table(_) => {
                nested_tables.insert(key.clone(), value.clone());
            }
            _ => {
                flat_table.insert(key.clone(), value.clone());
            }
        }
    }

    if !flat_table.is_empty() {
        let domain = match &prefix {
            Some(p) => p.clone(),
            None => String::new(),
        };
        dest.push((domain, flat_table));
    }

    for (key, value) in nested_tables {
        if let Value::Table(inner) = value {
            let new_prefix = match &prefix {
                Some(p) if !p.is_empty() => format!("{}.{}", p, key),
                _ => key.clone(),
            };
            flatten_domains(Some(new_prefix), &inner, dest);
        }
    }
}

/// Checks whether a given domain exists using `defaults read`.
fn check_domain_exists(full_domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Execute: defaults read <full_domain>
    let output = Command::new("defaults")
        .arg("read")
        .arg(full_domain)
        .output()?;
    if !output.status.success() {
        return Err(format!("Domain '{}' does not exist. Aborting.", full_domain).into());
    }
    Ok(())
}

/// Reads the config file, validates domains, and applies each default via `defaults write`.
fn apply_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        if verbose {
            println!("Config file not found at {:?}.", config_path);
            print!("Would you like to create an example config file? [y/N]: ");
        } else {
            print!(
                "{}[WARNING]{} No config found. Create example? [y/N]: ",
                YELLOW, RESET
            );
        }
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            create_example_config(&config_path, verbose)?;
            return Ok(());
        } else {
            return Err("No config file present. Exiting.".into());
        }
    }

    let content = fs::read_to_string(&config_path)?;
    let parsed: Value = content.parse::<Value>()?;
    let root_table = parsed
        .as_table()
        .ok_or("Invalid config format: expected table at top level")?;

    let mut domains: Vec<(String, toml::value::Table)> = Vec::new();
    for (key, value) in root_table {
        if let Value::Table(inner) = value {
            flatten_domains(Some(key.clone()), inner, &mut domains);
        }
    }

    for (domain, settings_table) in domains {
        let full_domain = format!("com.apple.{}", domain);

        check_domain_exists(&full_domain)?;

        // In verbose mode, print detailed output for each key.
        for (key, value) in settings_table {
            let (flag, value_str) = match value {
                Value::Boolean(b) => ("-bool", b.to_string()),
                Value::Integer(i) => ("-int", i.to_string()),
                Value::Float(f) => ("-float", f.to_string()),
                Value::String(s) => ("-string", s.clone()),
                _ => {
                    return Err(format!(
                        "Unsupported type for key '{}' in domain '{}'",
                        key, domain
                    )
                    .into());
                }
            };

            if verbose {
                println!(
                    "Applying: defaults write {} \"{}\" {} \"{}\"",
                    full_domain, key, flag, value_str
                );
            }

            let output = Command::new("defaults")
                .arg("write")
                .arg(&full_domain)
                .arg(&key)
                .arg(flag)
                .arg(&value_str)
                .output()?;

            if !output.status.success() {
                eprintln!(
                    "{}[ERROR] Failed to apply setting '{}' for {}.{}",
                    RED, key, full_domain, RESET
                );
            } else if verbose {
                println!(
                    "{}[SUCCESS] Applied setting '{}' for {}.{}",
                    GREEN, key, full_domain, RESET
                );
            }
        }
        if !verbose {
            // Minimal output per domain
            println!("Updated: {}", full_domain);
        }
    }
    Ok(())
}

/// Reads the config file, validates domains, and unapplies each default via `defaults delete`.
fn unapply_defaults(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Err(format!("Config file not found at {:?}.", config_path).into());
    }

    let content = fs::read_to_string(&config_path)?;
    let parsed: Value = content.parse::<Value>()?;
    let root_table = parsed
        .as_table()
        .ok_or("Invalid config format: expected table at top level")?;

    let mut domains: Vec<(String, toml::value::Table)> = Vec::new();
    for (key, value) in root_table {
        if let Value::Table(inner) = value {
            flatten_domains(Some(key.clone()), inner, &mut domains);
        }
    }

    for (domain, settings_table) in domains {
        let full_domain = format!("com.apple.{}", domain);
        check_domain_exists(&full_domain)?;

        for (key, _value) in settings_table {
            if verbose {
                println!("Unapplying: defaults delete {} \"{}\"", full_domain, key);
            }

            let output = Command::new("defaults")
                .arg("delete")
                .arg(&full_domain)
                .arg(&key)
                .output()?;

            if !output.status.success() {
                eprintln!(
                    "{}[ERROR] Failed to unapply setting '{}' for {}.{}",
                    RED, key, full_domain, RESET
                );
            } else if verbose {
                println!(
                    "{}[SUCCESS] Unapplied setting '{}' for {}.{}",
                    GREEN, key, full_domain, RESET
                );
            }
        }
        if !verbose {
            println!("Reverted: {}", full_domain);
        }
    }
    Ok(())
}

/// Deletes the configuration file if it exists.
fn delete_config(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    if config_path.exists() {
        // Ask for confirmation before deletion.
        if verbose {
            print!(
                "Are you sure you want to delete the configuration file at {:?}? [y/N]: ",
                config_path
            );
        } else {
            print!(
                "{}[WARNING]{} Confirm deletion of config at {:?}? [y/N]: ",
                YELLOW, RESET, config_path
            );
        }
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            if verbose {
                println!("{}[INFO] Deletion cancelled.{}", BLUE, RESET);
            } else {
                println!("Deletion cancelled.");
            }
            return Ok(());
        }
        fs::remove_file(&config_path)?;
        if verbose {
            println!(
                "{}[SUCCESS] Configuration file deleted from: {:?}{}",
                GREEN, config_path, RESET
            );
        } else {
            println!("🗑️ Config deleted from {:?}", config_path);
        }
    } else {
        if verbose {
            println!(
                "{}[SUCCESS] No configuration file found at: {:?}{}",
                GREEN, config_path, RESET
            );
        } else {
            println!("{}[WARNING]{} No config file to delete.", YELLOW, RESET);
        }
    }
    Ok(())
}

/// Kills (restarts) Finder, Dock, and SystemUIServer to refresh settings.
fn restart_system_services(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    for service in &["Finder", "Dock", "SystemUIServer"] {
        let output = Command::new("killall").arg(service).output()?;
        if !output.status.success() {
            eprintln!(
                "{}[ERROR] Failed to restart {}.{} Try restarting manually.",
                RED, service, RESET
            );
        } else if verbose {
            println!("{}[SUCCESS] {} restarted.{}", GREEN, service, RESET);
        }
    }
    if !verbose {
        println!("System services restarted. You're good!");
    }
    Ok(())
}
