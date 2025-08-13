use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use tokio::fs;
use toml_edit::{Array, DocumentMut, Item, Table, Value};

use crate::{
    brew::{
        types::BrewListType,
        utils::{brew_list, ensure_brew},
    },
    cli::atomic::should_dry_run,
    commands::Runnable,
    config::{loader::load_config_mut, path::get_config_path},
    util::{
        io::confirm_action,
        logging::{GREEN, LogLevel, RESET, print_log},
    },
};

#[derive(Debug, Default, Args)]
pub struct BrewBackupCmd {
    /// Exclude dependencies from backup.
    #[arg(long)]
    pub no_deps: bool,
}

#[async_trait]
impl Runnable for BrewBackupCmd {
    async fn run(&self) -> Result<()> {
        let cfg_path = get_config_path().await;
        let dry_run = should_dry_run();
        let mut backup_no_deps = self.no_deps;

        // ensure brew install
        ensure_brew().await?;

        // init config
        let mut doc = if fs::try_exists(&cfg_path).await.unwrap() {
            load_config_mut(true).await?
        } else {
            print_log(
                LogLevel::Warning,
                "Config file does not exist. Creating new...",
            );
            DocumentMut::new()
        };

        // init brew table from config
        let brew_item = doc.entry("brew").or_insert(Item::Table(Table::new()));
        let brew_tbl = brew_item.as_table_mut().unwrap();

        // firstly remember the --no-deps value
        if self.no_deps {
            if brew_tbl
                .get("no_deps")
                .is_none_or(|x| !x.as_bool().unwrap())
            {
                print_log(
                    LogLevel::Info,
                    "Setting no_deps to true in config for later reads.",
                );
                brew_tbl["no_deps"] = Item::Value(Value::Boolean(toml_edit::Formatted::new(true)));
            } else {
                print_log(
                    LogLevel::Info,
                    "no_deps already found true in configuration, so not setting.",
                );
            }
        } else if brew_tbl
            .get("no_deps")
            .is_some_and(|x| x.as_bool().unwrap())
            && confirm_action("The previous backup was without dependencies. Do now too?")
        {
            backup_no_deps = true
        } else {
            brew_tbl["no_deps"] = Item::None;
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

        // build TOML arrays for formulae and casks
        let mut formula_arr = Array::new();
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
                        formula_arr.push(formula.as_str());
                    }
                }
            } else if dry_run {
                print_log(LogLevel::Dry, &format!("Would push {formula}"));
            } else {
                print_log(LogLevel::Info, &format!("Pushing {formula}"));
                formula_arr.push(formula.as_str());
            }
        }
        print_log(
            LogLevel::Info,
            &format!("{}Pushed {} formulae.{}", GREEN, formula_arr.len(), RESET),
        );
        brew_tbl["formulae"] = Item::Value(Value::Array(formula_arr));

        let mut cask_arr = Array::new();
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
                        cask_arr.push(cask.as_str());
                    }
                }
            } else if dry_run {
                print_log(LogLevel::Dry, &format!("Would push {cask}"));
            } else {
                print_log(LogLevel::Info, &format!("Pushed {cask} as a cask."));
                cask_arr.push(cask.as_str());
            }
        }
        print_log(
            LogLevel::Info,
            &format!("{}Pushed {} casks.{}", GREEN, cask_arr.len(), RESET),
        );
        brew_tbl["casks"] = Item::Value(Value::Array(cask_arr));

        // backup taps
        let mut taps_arr = Array::new();
        for tap in &taps {
            if dry_run {
                print_log(LogLevel::Dry, &format!("Would push {tap} as tap."));
            } else {
                print_log(LogLevel::Info, &format!("Pushed {tap} as a tap."));
                taps_arr.push(tap.as_str());
            }
        }
        print_log(
            LogLevel::Info,
            &format!("{}Pushed {} taps.{}", GREEN, taps_arr.len(), RESET),
        );
        brew_tbl["taps"] = Item::Value(Value::Array(taps_arr));

        // write backup
        if !dry_run {
            fs::create_dir_all(cfg_path.parent().unwrap()).await?;
            fs::write(&cfg_path, doc.to_string()).await?;

            print_log(LogLevel::Info, &format!("Backup saved to {cfg_path:?}"));
            print_log(
                LogLevel::Fruitful,
                &format!("Done! You can find the backup in your config file location {cfg_path:?}"),
            );
        } else {
            print_log(
                LogLevel::Info,
                &format!("Backup would be saved to {cfg_path:?}"),
            );
        }

        Ok(())
    }
}
