use anyhow::{Result, anyhow};
use std::fs;

use crate::{
    config::loader::get_config_path,
    util::{
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};

pub fn run(verbose: bool, force: bool) -> Result<()> {
    let config_path = get_config_path();

    if config_path.exists() && !force {
        print_log(
            LogLevel::Warning,
            &format!("Configuration file already exists at {:?}", config_path),
        );
        if !confirm_action("Do you want to overwrite it?")? {
            return Err(anyhow!("Configuration initialization aborted."));
        }
    }

    // ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // default TOML template
    let default_cfg = r#"# Generated with cutler
# See https://github.com/hitblast/cutler for more examples

[menuextra.clock]
FlashDateSeparators = true
DateFormat            = "\"HH:mm:ss\""
Show24Hour            = true
ShowAMPM              = false
ShowDate              = 2
ShowDayOfWeek         = false
ShowSeconds           = true

[finder]
AppleShowAllFiles     = true
CreateDesktop         = false
ShowPathbar           = true
FXRemoveOldTrashItems = true

[AppleMultitouchTrackpad]
FirstClickThreshold   = 0
TrackpadThreeFingerDrag = true

[dock]
tilesize             = 50
autohide             = true
magnification        = false
orientation          = "right"
mineffect            = "suck"
autohide-delay       = 0
autohide-time-modifier = 0.6
expose-group-apps    = true

[NSGlobalDomain.com.apple.keyboard]
fnState = false

# External commands (uncomment / customize as needed)
# [vars]
# hostname = "my-macbook"
# 
# [commands.hostname]
# cmd  = "scutil --set ComputerName $hostname"
# sudo = true
"#;

    fs::write(&config_path, default_cfg)
        .map_err(|e| anyhow!("Failed to write configuration to {:?}: {}", config_path, e))?;

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
