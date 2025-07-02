use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, preferences::Preferences};
use tokio::fs;

use crate::{
    commands::Runnable,
    config::loader::{get_config_path, load_config},
    domains::{collect, effective, read_current},
    snapshot::state::get_snapshot_path,
    util::{
        globals::should_dry_run,
        io::{confirm_action, restart_system_services},
        logging::{LogLevel, print_log},
    },
};

#[derive(Args, Debug)]
pub struct ResetCmd {
    /// Forcefully reset configuration.
    #[arg(short, long)]
    force: bool,
}

#[async_trait]
impl Runnable for ResetCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path();
        if !config_path.exists() {
            bail!("No config file found. Please run `cutler init` first, or create a config file.");
        }

        let dry_run = should_dry_run();

        print_log(
            LogLevel::Warning,
            "This will DELETE all settings defined in your config file.",
        );
        print_log(
            LogLevel::Warning,
            "Settings will be reset to macOS defaults, not to their previous values.",
        );

        if !self.force && !confirm_action("Are you sure you want to continue?")? {
            return Ok(());
        }

        let toml = load_config(&config_path, true).await?;
        let domains = collect(&toml)?;

        for (domain, table) in domains {
            for (key, _) in table {
                let (eff_dom, eff_key) = effective(&domain, &key);

                // only delete it if currently set
                if read_current(&eff_dom, &eff_key).await.is_some() {
                    let domain_obj = if eff_dom == "NSGlobalDomain" {
                        Domain::Global
                    } else if let Some(rest) = eff_dom.strip_prefix("com.apple.") {
                        Domain::User(format!("com.apple.{rest}"))
                    } else {
                        Domain::User(eff_dom.clone())
                    };

                    if dry_run {
                        print_log(
                            LogLevel::Dry,
                            &format!("Would reset {eff_dom}.{eff_key} to system default"),
                        );
                    } else {
                        match Preferences::delete(domain_obj, Some(&eff_key)).await {
                            Ok(_) => {
                                print_log(
                                    LogLevel::Info,
                                    &format!("Reset {eff_dom}.{eff_key} to system default"),
                                );
                            }
                            Err(e) => {
                                print_log(
                                    LogLevel::Error,
                                    &format!("Failed to reset {eff_dom}.{eff_key}: {e}"),
                                );
                            }
                        }
                    }
                } else {
                    print_log(
                        LogLevel::Info,
                        &format!("Skipping {eff_dom}.{eff_key} (not set)"),
                    );
                }
            }
        }

        // remove snapshot if present
        let snap_path = get_snapshot_path();
        if snap_path.exists() {
            if dry_run {
                print_log(
                    LogLevel::Dry,
                    &format!("Would remove snapshot at {snap_path:?}"),
                );
            } else if let Err(e) = fs::remove_file(&snap_path).await {
                print_log(
                    LogLevel::Warning,
                    &format!("Failed to remove snapshot: {e}"),
                );
            } else {
                print_log(
                    LogLevel::Info,
                    &format!("Removed snapshot at {snap_path:?}"),
                );
            }
        }

        print_log(
            LogLevel::Fruitful,
            "Reset complete. All configured settings have been removed.",
        );

        // restart system services if requested
        restart_system_services().await?;

        Ok(())
    }
}
