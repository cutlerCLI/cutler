use crate::{
    config::loader::{get_config_path, load_config},
    external::runner,
    snapshot::state::Snapshot,
    util::{
        io::confirm_action,
        logging::{LogLevel, print_log},
    },
};
use anyhow::Result;
use tokio::task;

pub async fn run(which: Option<String>, verbose: bool, dry_run: bool) -> Result<()> {
    let config_path = get_config_path();
    if !config_path.exists() {
        print_log(
            LogLevel::Info,
            &format!("Config not found at {:?}", config_path),
        );
        if confirm_action("Would you like to create a new configuration?")? {
            // reuse init
            super::init::run(verbose, false).await?;

            print_log(
                LogLevel::Info,
                "Configuration created. Please review it before running external commands.",
            );
            return Ok(());
        } else {
            anyhow::bail!("No config file present. Exiting.");
        }
    }

    // load & parse config
    let toml = load_config(&config_path).await?;

    // load or init snapshot
    let snap_path = crate::snapshot::state::get_snapshot_path();
    let mut snapshot = if snap_path.exists() {
        Snapshot::load(&snap_path).unwrap_or_else(|e| {
            print_log(
                LogLevel::Warning,
                &format!(
                    "Could not load existing snapshot: {}. Creating a new one.",
                    e
                ),
            );
            Snapshot::new()
        })
    } else {
        Snapshot::new()
    };

    print_log(
        LogLevel::Info,
        "Executing only external commands (skipping defaults)",
    );

    // record external commands into the snapshot
    snapshot.external = runner::extract(&toml);

    // save the snapshot before executing
    if dry_run {
        print_log(
            LogLevel::Info,
            &format!("Dry-run: Would save snapshot to {:?}", snap_path),
        );
    } else {
        // save snapshot off the async runtime
        let snap = snapshot;
        let path = snap_path.clone();
        task::spawn_blocking(move || snap.save(&path)).await??;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Snapshot updated at {:?} (external commands)", snap_path),
            );
        }
    }

    if let Some(cmd_name) = which {
        runner::run_one(&toml, &cmd_name, verbose, dry_run).await?;
    } else {
        runner::run_all(&toml, verbose, dry_run).await?;
    }

    if !verbose && !dry_run {
        println!("\n🍎 External commands executed successfully.");
    }

    Ok(())
}
