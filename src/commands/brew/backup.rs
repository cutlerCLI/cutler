// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

use crate::{
    brew::{
        core::{brew_list, ensure_brew},
        types::BrewListType,
    },
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{core::Config, path::get_config_path},
    log_cute, log_dry, log_info, log_warn,
    util::io::confirm,
};

#[derive(Debug, Args)]
pub struct BrewBackupCmd {
    /// Exclude dependencies from backup.
    #[arg(long)]
    no_deps: bool,
}

#[async_trait]
impl Runnable for BrewBackupCmd {
    async fn run(&self) -> Result<()> {
        let dry_run = should_dry_run();
        let mut backup_no_deps = self.no_deps;

        // ensure brew install
        ensure_brew().await?;

        // init config
        let config_path = get_config_path().await?;
        let mut config = Config::new(config_path);

        config = if config.path.try_exists()? {
            config.load(true).await?
        } else {
            log_warn!("Config file does not exist. Creating new...",);
            config
        };

        // Prepare Brew struct for backup
        let mut brew = config.brew.clone().unwrap_or_default();

        // firstly remember the --no-deps value
        if self.no_deps {
            if brew.no_deps != Some(true) {
                log_info!("Setting no_deps to true in config for later reads.",);
                brew.no_deps = Some(true);
            } else {
                log_info!("no_deps already found true in configuration, so not setting.",);
            }
        } else if brew.no_deps == Some(true)
            && confirm("The previous backup was without dependencies. Do now too?")
        {
            backup_no_deps = true
        } else {
            brew.no_deps = None;
        }

        // load deps into memory for comparison
        // this will also be reused for later comparisons
        let deps = if backup_no_deps {
            brew_list(BrewListType::Dependency, false).await?
        } else {
            vec![]
        };

        // load the formulae, casks and taps list from the `brew` command
        // flattening is `false` since we want all names to be forced to --full-name
        let formulas = brew_list(BrewListType::Formula, false).await?;
        let casks = brew_list(BrewListType::Cask, false).await?;
        let taps = brew_list(BrewListType::Tap, false).await?;

        // build formulae and casks arrays
        let mut formula_arr = Vec::new();
        for formula in &formulas {
            if backup_no_deps {
                if !deps.contains(formula) {
                    if dry_run {
                        log_dry!("Would push {formula} as a manually installed formula.",);
                    } else {
                        log_info!("Pushing {formula} as a manually installed formula.",);
                        formula_arr.push(formula.clone());
                    }
                }
            } else if dry_run {
                log_dry!("Would push {formula}");
            } else {
                log_info!("Pushing {formula}");
                formula_arr.push(formula.clone());
            }
        }
        log_info!("Pushed {} formulae.", formula_arr.len());
        brew.formulae = Some(formula_arr);

        let mut cask_arr = Vec::new();
        for cask in &casks {
            if backup_no_deps {
                if !deps.contains(cask) {
                    if dry_run {
                        log_dry!("Would push {cask} as a manually installed cask.",);
                    } else {
                        log_info!("Pushing {cask} as a manually installed cask.",);
                        cask_arr.push(cask.clone());
                    }
                }
            } else if dry_run {
                log_dry!("Would push {cask}");
            } else {
                log_info!("Pushed {cask} as a cask.");
                cask_arr.push(cask.clone());
            }
        }
        log_info!("Pushed {} casks.", cask_arr.len());
        brew.casks = Some(cask_arr);

        // backup taps
        let mut taps_arr = Vec::new();
        for tap in &taps {
            if dry_run {
                log_dry!("Would push {tap} as tap.");
            } else {
                log_info!("Pushed {tap} as a tap.");
                taps_arr.push(tap.clone());
            }
        }
        log_info!("Pushed {} taps.", taps_arr.len());
        brew.taps = Some(taps_arr);

        // update config
        config.brew = Some(brew);

        // write backup
        if !dry_run {
            config.save(None).await?;

            log_cute!("Done!");
        } else {
            log_info!("Backup would be saved to {:?}", config.path,);
        }

        Ok(())
    }
}
