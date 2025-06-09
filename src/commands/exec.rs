use crate::commands::{GlobalArgs, Runnable};
use crate::{
    config::loader::load_config,
    external::runner,
    snapshot::state::Snapshot,
    util::logging::{LogLevel, print_log},
};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

/// Run only the external commands written in the config file.
#[derive(Args, Debug)]
pub struct ExecCmd {
    /// Provide a command name to execute if you only want to run it specifically.
    #[arg(value_name = "NAME")]
    pub name: Option<String>,
}

#[async_trait]
impl Runnable for ExecCmd {
    async fn run(&self, g: &GlobalArgs) -> Result<()> {
        let verbose = g.verbose;
        let dry_run = g.dry_run;
        let quiet = g.quiet;

        let config_path_opt = crate::util::config::ensure_config_exists_or_init(g).await?;
        let config_path = match config_path_opt {
            Some(path) => path,
            None => anyhow::bail!("Aborted."),
        };

        // load & parse config
        let toml = load_config(&config_path).await?;

        // load or init snapshot
        let snap_path = crate::snapshot::state::get_snapshot_path();
        let mut snapshot = if snap_path.exists() {
            Snapshot::load(&snap_path).await.unwrap_or_else(|e| {
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
            if !quiet {
                print_log(
                    LogLevel::Dry,
                    &format!("Would save snapshot to {:?}", snap_path),
                );
            }
        } else {
            let snap = snapshot;
            let path = snap_path.clone();
            snap.save(&path).await?;

            if verbose {
                print_log(
                    LogLevel::Success,
                    &format!("Snapshot updated at {:?} (external commands)", snap_path),
                );
            }
        }

        if let Some(cmd_name) = &self.name {
            runner::run_one(&toml, cmd_name, verbose, dry_run).await?;
        } else {
            runner::run_all(&toml, verbose, dry_run).await?;
        }

        if !verbose && !dry_run && !quiet {
            println!("\n🍎 External commands executed successfully.");
        }

        Ok(())
    }
}
