use std::io::{self, Write};
use std::process::Command;

use crate::util::logging::{LogLevel, print_log};

/// Ask “Y/N?”; returns true only if the user types “y” or “Y”
pub fn confirm_action(prompt: &str) -> io::Result<bool> {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(matches!(buf.trim().to_lowercase().as_str(), "y"))
}

/// Restart Finder, Dock, SystemUIServer so defaults take effect.
pub fn restart_system_services(verbose: bool, dry_run: bool) -> Result<(), anyhow::Error> {
    const SERVICES: &[&str] = &["Finder", "Dock", "SystemUIServer"];
    for svc in SERVICES {
        if dry_run {
            if verbose {
                print_log(LogLevel::Info, &format!("Would restart {}", svc));
            }
        } else {
            let out = Command::new("killall").arg(svc).output()?;
            if !out.status.success() {
                print_log(LogLevel::Error, &format!("Failed to restart {}", svc));
            } else if verbose {
                print_log(LogLevel::Success, &format!("{} restarted", svc));
            }
        }
    }
    if !verbose && !dry_run {
        println!("\n🍎 Done. System services restarted.");
    } else if dry_run {
        println!("\n🍎 Dry‑run: would restart services.");
    }
    Ok(())
}
