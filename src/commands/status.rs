// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    brew::{
        core::{brew_is_installed, diff_brew},
        types::BrewDiff,
    },
    commands::Runnable,
    config::core::Config,
    domains::{collect, effective, read_current},
    log_cute, log_err, log_info, log_warn,
    util::logging::{BOLD, GREEN, RED, RESET},
};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use std::collections::{HashMap, HashSet};

#[derive(Args, Debug)]
pub struct StatusCmd {
    // Disables Homebrew state check.
    #[arg(long)]
    no_brew: bool,
}

#[async_trait]
impl Runnable for StatusCmd {
    async fn run(&self) -> Result<()> {
        let config = Config::load(false).await?;
        let domains = collect(&config)?;

        // flatten all settings into a list
        let entries: Vec<(String, String, toml::Value)> = domains
            .into_iter()
            .flat_map(|(domain, table)| {
                table
                    .into_iter()
                    .map(move |(key, value)| (domain.clone(), key.clone(), value.clone()))
            })
            .collect();

        // preference check
        {
            let mut outcomes = Vec::with_capacity(entries.len());
            let mut domain_has_diff = HashMap::new();

            // let the checks begin!
            for (domain, key, value) in entries.iter() {
                let (eff_dom, eff_key) = effective(domain, key);

                let current_pref = read_current(&eff_dom, &eff_key).await;
                let desired_pref = crate::domains::convert::toml_to_prefvalue(value)?;
                
                let (current_str, is_diff) = match &current_pref {
                    Some(current) => {
                        let diff = current != &desired_pref;
                        (current.to_string(), diff)
                    }
                    None => ("Not set".to_string(), true),
                };
                let desired_str = desired_pref.to_string();

                outcomes.push((
                    eff_dom.clone(),
                    eff_key,
                    desired_str.clone(),
                    current_str.clone(),
                    is_diff,
                ));

                // set to false only if it hasn't been set to true once
                // we use it later for LogLevel::Warning over domains which have at least one diff
                if is_diff {
                    domain_has_diff.insert(eff_dom.clone(), true);
                } else {
                    domain_has_diff.entry(eff_dom.clone()).or_insert(false);
                }
            }

            // keep track of printed domains so that they're only printed once
            // the iterable keeps the domain key-value pairs sequentially so this is a plus
            let mut printed_domains = HashSet::new();
            let mut any_diff = false;

            for (eff_dom, eff_key, desired, current, is_diff) in outcomes {
                if !printed_domains.contains(&eff_dom) {
                    if *domain_has_diff.get(&eff_dom).unwrap_or(&false) {
                        log_warn!("{BOLD}{eff_dom}{RESET}");
                    } else {
                        log_info!("{BOLD}{eff_dom}{RESET}");
                    }
                    printed_domains.insert(eff_dom.clone());
                }

                if is_diff {
                    if !any_diff {
                        any_diff = true
                    }
                    log_warn!(
                        "  {eff_key}: should be {RED}{desired}{RESET} (now: {RED}{current}{RESET})",
                    );
                } else {
                    log_info!("  {GREEN}[Matched]{RESET} {eff_key}: {current}",);
                }
            }

            if any_diff {
                log_warn!("Preferences diverged. Run `cutler apply` to apply changes.",);
            } else {
                log_cute!("System preferences are on sync.");
            }
        }

        // brew status check
        {
            let toml_brew = config.clone();
            let no_brew = self.no_brew;

            if !no_brew && let Some(brew_val) = toml_brew.brew {
                log_info!("Homebrew status:");

                // ensure homebrew is installed (skip if not)
                if !brew_is_installed().await {
                    log_warn!("Homebrew not available in $PATH, skipping status check for it.",);
                } else {
                    match diff_brew(brew_val).await {
                        Ok(BrewDiff {
                            missing_formulae,
                            extra_formulae,
                            missing_casks,
                            extra_casks,
                            missing_taps,
                            extra_taps,
                        }) => {
                            let mut any_diff = false;

                            // Use a single array of tuples to reduce repeated code
                            let brew_checks = [
                                ("Formulae missing", &missing_formulae),
                                ("Extra formulae installed", &extra_formulae),
                                ("Casks missing", &missing_casks),
                                ("Extra casks installed", &extra_casks),
                                ("Missing taps", &missing_taps),
                                ("Extra taps", &extra_taps),
                            ];

                            for (label, items) in brew_checks.iter() {
                                if !items.is_empty() {
                                    any_diff = true;
                                    log_warn!("{BOLD}{label}:{RESET} {}", items.join(", "));
                                }
                            }

                            if any_diff {
                                log_warn!("Homebrew diverged.",);

                                if !missing_casks.is_empty()
                                    || !missing_formulae.is_empty()
                                    || !missing_taps.is_empty()
                                {
                                    log_warn!(
                                        "Run `cutler brew install` to install missing software."
                                    )
                                }
                                if !extra_casks.is_empty()
                                    || !extra_formulae.is_empty()
                                    || !extra_taps.is_empty()
                                {
                                    log_warn!("Run `cutler brew backup` to backup extra software.")
                                }
                            } else {
                                log_cute!("Homebrew status on sync.");
                            }
                        }
                        Err(e) => {
                            log_err!("Could not check Homebrew status: {e}",);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
