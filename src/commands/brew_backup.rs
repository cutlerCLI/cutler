use tokio::fs;

use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use toml_edit::{Array, DocumentMut, Item, Table, Value};

use crate::{
    brew::utils::{
        brew_list, brew_list_taps, disable_auto_update, ensure_brew, restore_auto_update,
    },
    commands::{GlobalArgs, Runnable},
    config::get_config_path,
    util::logging::{LogLevel, print_log},
};

#[derive(Debug, Default, Args)]
pub struct BrewBackupCmd {
    /// Exclude dependencies from backup
    #[arg(long)]
    pub no_deps: bool,
}

#[async_trait]
impl Runnable for BrewBackupCmd {
    async fn run(&self, g: &GlobalArgs) -> Result<()> {
        let cfg_path = get_config_path();

        let dry_run = g.dry_run;
        let quiet = g.quiet;
        let verbose = g.verbose;

        // disable auto-update
        let prev = disable_auto_update();

        // ensure brew install
        ensure_brew(dry_run).await?;

        let formulas = brew_list(&["list", "--formula"]).await?;
        let casks = brew_list(&["list", "--cask"]).await?;
        let deps = brew_list(&["list", "--installed-as-dependency"]).await?;

        // fetch taps using the shared utility
        let taps = brew_list_taps().await?;

        if dry_run {
            if !quiet {
                print_log(
                    LogLevel::Dry,
                    &format!(
                        "Would backup {} formulae and {} casks",
                        formulas.len(),
                        casks.len()
                    ),
                );
            }
            return Ok(());
        }

        let mut doc = if cfg_path.exists() {
            let text = fs::read_to_string(&cfg_path).await?;
            text.parse::<DocumentMut>()
                .context("Failed to parse config TOML")?
        } else {
            DocumentMut::new()
        };

        let brew_item = doc.entry("brew").or_insert(Item::Table(Table::new()));
        let brew_tbl = brew_item.as_table_mut().unwrap();

        // build TOML arrays for formulae and casks
        let mut formula_arr = Array::new();
        for formula in &formulas {
            if self.no_deps {
                if !deps.contains(formula) {
                    if verbose && !quiet {
                        print_log(
                            LogLevel::Info,
                            &format!("Pushing {} as a manually installed formula.", formula),
                        );
                    }
                    formula_arr.push(formula.as_str());
                }
            } else {
                if verbose && !quiet {
                    print_log(LogLevel::Info, &format!("Pushing {}", formula));
                }
                formula_arr.push(formula.as_str());
            }
        }
        if verbose && !quiet {
            print_log(
                LogLevel::Info,
                &format!("Pushed {} formulae.", formula_arr.len()),
            );
        }
        brew_tbl["formulae"] = Item::Value(Value::Array(formula_arr));

        let mut cask_arr = Array::new();
        for cask in &casks {
            if verbose && !quiet {
                print_log(LogLevel::Info, &format!("Pushed {} as a cask.", cask));
            }
            cask_arr.push(cask.as_str());
        }
        if verbose && !quiet {
            print_log(LogLevel::Info, &format!("Pushed {} casks.", cask_arr.len()));
        }
        brew_tbl["casks"] = Item::Value(Value::Array(cask_arr));

        // backup taps
        let mut taps_arr = Array::new();
        for tap in &taps {
            if verbose && !quiet {
                print_log(LogLevel::Info, &format!("Pushed {} as a tap.", tap));
            }
            taps_arr.push(tap.as_str());
        }
        if verbose && !quiet {
            print_log(LogLevel::Info, &format!("Pushed {} taps.", taps_arr.len()));
        }
        brew_tbl["taps"] = Item::Value(Value::Array(taps_arr));

        // give length of both lists in verbose, and let the user know about config location
        if verbose && !quiet {
            print_log(LogLevel::Info, &format!("Writing backup to {:?}", cfg_path));
        }
        fs::write(&cfg_path, doc.to_string()).await?;

        // output message
        if verbose && !quiet {
            print_log(
                LogLevel::Success,
                &format!("Backup saved to {:?}", cfg_path),
            );
        } else if !quiet {
            println!(
                "\n🍎 Done! You can find the backup in your config file location {:?}",
                cfg_path
            );
        }

        restore_auto_update(prev);
        Ok(())
    }
}
