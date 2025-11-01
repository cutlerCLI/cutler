// SPDX-License-Identifier: GPL-3.0-or-later

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
    config::core::Config,
    util::{
        io::confirm,
        logging::{LogLevel, print_log},
    },
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
        let mut config = if Config::is_loadable().await {
            Config::load(true).await?
        } else {
            print_log(
                LogLevel::Warning,
                "Config file does not exist. Creating new...",
            );
            Config::new().await
        };

        // Prepare Brew struct for backup
        let mut brew = config.brew.clone().unwrap_or_default();

        // firstly remember the --no-deps value
        if self.no_deps {
            if brew.no_deps != Some(true) {
                print_log(
                    LogLevel::Info,
                    "Setting no_deps to true in config for later reads.",
                );
                brew.no_deps = Some(true);
            } else {
                print_log(
                    LogLevel::Info,
                    "no_deps already found true in configuration, so not setting.",
                );
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
            brew_list(BrewListType::Dependency).await?
        } else {
            vec![]
        };

        // load the formulae, casks and taps list from the `brew` command
        let formulas = brew_list(BrewListType::Formula).await?;
        let casks = brew_list(BrewListType::Cask).await?;
        let taps = brew_list(BrewListType::Tap).await?;

        // build formulae and casks arrays
        let mut formula_arr = Vec::new();
        for formula in &formulas {
            if backup_no_deps {
                if !deps.contains(formula) {
                    if dry_run {
                        print_log(
                            LogLevel::Dry,
                            &format!("Would push {formula} as a manually installed formula."),
                        );
                    } else {
                        print_log(
                            LogLevel::Info,
                            &format!("Pushing {formula} as a manually installed formula."),
                        );
                        formula_arr.push(formula.clone());
                    }
                }
            } else if dry_run {
                print_log(LogLevel::Dry, &format!("Would push {formula}"));
            } else {
                print_log(LogLevel::Info, &format!("Pushing {formula}"));
                formula_arr.push(formula.clone());
            }
        }
        print_log(
            LogLevel::Info,
            &format!("Pushed {} formulae.", formula_arr.len()),
        );
        brew.formulae = Some(formula_arr);

        let mut cask_arr = Vec::new();
        for cask in &casks {
            if backup_no_deps {
                if !deps.contains(cask) {
                    if dry_run {
                        print_log(
                            LogLevel::Dry,
                            &format!("Would push {cask} as a manually installed cask."),
                        );
                    } else {
                        print_log(
                            LogLevel::Info,
                            &format!("Pushing {cask} as a manually installed cask."),
                        );
                        cask_arr.push(cask.clone());
                    }
                }
            } else if dry_run {
                print_log(LogLevel::Dry, &format!("Would push {cask}"));
            } else {
                print_log(LogLevel::Info, &format!("Pushed {cask} as a cask."));
                cask_arr.push(cask.clone());
            }
        }
        print_log(LogLevel::Info, &format!("Pushed {} casks.", cask_arr.len()));
        brew.casks = Some(cask_arr);

        // backup taps
        let mut taps_arr = Vec::new();
        for tap in &taps {
            if dry_run {
                print_log(LogLevel::Dry, &format!("Would push {tap} as tap."));
            } else {
                print_log(LogLevel::Info, &format!("Pushed {tap} as a tap."));
                taps_arr.push(tap.clone());
            }
        }
        print_log(LogLevel::Info, &format!("Pushed {} taps.", taps_arr.len()));
        brew.taps = Some(taps_arr);

        // update config
        config.brew = Some(brew);

        // write backup
        if !dry_run {
            config.save().await?;

            print_log(
                LogLevel::Info,
                &format!("Backup saved to {:?}", config.path),
            );
            print_log(
                LogLevel::Fruitful,
                &format!(
                    "Done! You can find the backup in your config file location {:?}",
                    config.path
                ),
            );
        } else {
            print_log(
                LogLevel::Info,
                &format!("Backup would be saved to {:?}", config.path),
            );
        }

        Ok(())
    }
}
