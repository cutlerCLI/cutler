use crate::commands::Runnable;
use crate::util::config::ensure_config_exists_or_init;
use crate::util::globals::should_dry_run;
use crate::{
    config::loader::load_config,
    exec::runner,
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
    async fn run(&self) -> Result<()> {
        let dry_run = should_dry_run();

        let config_path_opt = ensure_config_exists_or_init().await?;
        let config_path = match config_path_opt {
            Some(path) => path,
            None => anyhow::bail!("Aborted."),
        };

        // load & parse config
        let toml = load_config(&config_path, true).await?;

        if let Some(cmd_name) = &self.name {
            runner::run_one(&toml, cmd_name).await?;
        } else {
            runner::run_all(&toml).await?;
        }

        if !dry_run {
            print_log(
                LogLevel::Fruitful,
                "External commands executed successfully.",
            );
        }

        Ok(())
    }
}
