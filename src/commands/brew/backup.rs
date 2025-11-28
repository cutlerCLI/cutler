// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use toml_edit::{Array, DocumentMut, Item, Table, value};

use crate::{
    brew::{
        core::{brew_list, ensure_brew},
        types::BrewListType,
    },
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{Config, ConfigCoreMethods},
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
    fn needs_sudo(&self) -> bool {
        false
    }

    async fn run(&self, conf: &Config) -> Result<()> {
        let dry_run = should_dry_run();
        let mut backup_no_deps = self.no_deps;

        // ensure brew install
        ensure_brew().await?;

        // init config
        let mut doc = if let Ok(doc) = conf.load_as_mut(true).await {
            doc
        } else {
            log_warn!("Configuration does not exist; a new one will be created.");
            DocumentMut::new()
        };

        let brew_item = doc.entry("brew").or_insert(Item::Table(Table::new()));
        let brew_tbl = if let Some(brew_tbl) = brew_item.as_table_mut() {
            brew_tbl
        } else {
            &mut Table::new()
        };

        // firstly remember the --no-deps value
        let no_deps = brew_tbl
            .get("no_deps")
            .and_then(toml_edit::Item::as_bool)
            .unwrap_or(false);

        if self.no_deps {
            if no_deps {
                log_info!("Setting no_deps to true in config for later reads.",);
                brew_tbl["no_deps"] = value(false);
            } else {
                log_info!("no_deps already found true in configuration, so not setting.",);
            }
        } else if no_deps && confirm("The previous backup was without dependencies. Do now too?") {
            backup_no_deps = true;
        } else {
            brew_tbl["no_deps"] = Item::None;
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
        let mut formula_arr = Array::new();
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
        brew_tbl["formulae"] = value(formula_arr);

        let mut cask_arr = Array::new();
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
        brew_tbl["casks"] = value(cask_arr);

        // backup taps
        let mut taps_arr = Array::new();
        for tap in &taps {
            if dry_run {
                log_dry!("Would push {tap} as tap.");
            } else {
                log_info!("Pushed {tap} as a tap.");
                taps_arr.push(tap.clone());
            }
        }
        log_info!("Pushed {} taps.", taps_arr.len());
        brew_tbl["taps"] = value(taps_arr);

        // write backup
        if dry_run {
            log_info!("Backup would be saved to {:?}", &conf.path);
        } else {
            doc.save(&conf.path).await?;

            log_cute!("Done!");
        }

        Ok(())
    }
}
