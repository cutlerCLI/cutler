// SPDX-License-Identifier: Apache-2.0

use crate::{
    brew::{
        types::BrewDiff,
        utils::{compare_brew_state, is_brew_installed},
    },
    commands::Runnable,
    config::loader::load_config,
    domains::{collect, convert::normalize, effective, read_current},
    util::logging::{BOLD, GREEN, LogLevel, RED, RESET, print_log},
};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use std::collections::HashSet;

#[derive(Args, Debug)]
pub struct StatusCmd {
    // Disables Homebrew state check.
    #[arg(long)]
    pub no_brew: bool,
}

#[derive(PartialEq)]
enum StatusType {
    BothGood,
    BrewGoodOnly,
    PrefsGood,
    PrefsGoodOnly,
    NoneGood,
}

#[async_trait]
impl Runnable for StatusCmd {
    async fn run(&self) -> Result<()> {
        let toml = load_config(false).await?;
        let domains = collect(&toml)?;

        // status var
        let mut status: StatusType = StatusType::NoneGood;

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
            let entries_pref = entries.clone();

            // collect results
            let mut outcomes = Vec::with_capacity(entries_pref.len());
            for (domain, key, value) in entries_pref.iter() {
                let (eff_dom, eff_key) = effective(domain, key);

                let current = read_current(&eff_dom, &eff_key)
                    .await
                    .unwrap_or_else(|| "Not set".into());
                let desired = normalize(value);
                let is_diff = current != desired;

                outcomes.push((eff_dom, eff_key, desired, current, is_diff));
            }

            let mut printed_domains = HashSet::new();

            let mut any_diff = false;
            for (eff_dom, eff_key, desired, current, is_diff) in outcomes {
                if !printed_domains.contains(&eff_dom) {
                    let loglevel = if is_diff {
                        LogLevel::Warning
                    } else {
                        LogLevel::Info
                    };

                    print_log(loglevel, &format!("{BOLD}{eff_dom}{RESET}"));
                    printed_domains.insert(eff_dom.clone());
                }
                if is_diff {
                    if !any_diff {
                        any_diff = true
                    }
                    print_log(
                        LogLevel::Warning,
                        &format!(
                            "  {eff_key}: should be {RED}{desired}{RESET} (now: {RED}{current}{RESET})",
                        ),
                    );
                } else {
                    print_log(
                        LogLevel::Info,
                        &format!("  {GREEN}[Matched]{RESET} {eff_key}: {current}"),
                    );
                }
            }

            if !any_diff {
                status = StatusType::PrefsGood
            }
        }

        // brew status check
        {
            let toml_brew = toml.clone();
            let no_brew = self.no_brew;

            if !no_brew && let Some(brew_val) = toml_brew.get("brew").and_then(|v| v.as_table()) {
                print_log(LogLevel::Info, "Homebrew status:");

                // ensure homebrew is installed (skip if not)
                if !is_brew_installed().await {
                    print_log(
                        LogLevel::Warning,
                        "Homebrew not available in PATH, skipping status check for it.",
                    );
                } else {
                    match compare_brew_state(brew_val).await {
                        Ok(BrewDiff {
                            missing_formulae,
                            extra_formulae,
                            missing_casks,
                            extra_casks,
                            missing_taps,
                            extra_taps,
                        }) => {
                            let mut any_brew_diff = false;

                            if !missing_formulae.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!(
                                        "{BOLD}Formulae missing:{RESET} {}",
                                        missing_formulae.join(", ")
                                    ),
                                );
                            }
                            if !extra_formulae.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!(
                                        "{BOLD}Extra formulae installed:{RESET} {}",
                                        extra_formulae.join(", ")
                                    ),
                                );
                            }
                            if !missing_casks.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!(
                                        "{BOLD}Casks missing:{RESET} {}",
                                        missing_casks.join(", ")
                                    ),
                                );
                            }
                            if !extra_casks.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!(
                                        "{BOLD}Extra casks installed:{RESET} {}",
                                        extra_casks.join(", ")
                                    ),
                                );
                            }
                            if !missing_taps.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!(
                                        "{BOLD}Missing taps:{RESET} {}",
                                        missing_taps.join(", ")
                                    ),
                                );
                            }
                            if !extra_taps.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!("{BOLD}Extra taps:{RESET} {}", extra_taps.join(", ")),
                                );
                            }

                            if !any_brew_diff {
                                if status == StatusType::PrefsGood {
                                    status = StatusType::BothGood
                                } else {
                                    status = StatusType::BrewGoodOnly
                                };
                            } else if status == StatusType::PrefsGood {
                                status = StatusType::PrefsGoodOnly
                            }
                        }
                        Err(e) => {
                            print_log(
                                LogLevel::Error,
                                &format!("Could not check Homebrew status: {e}"),
                            );
                        }
                    }
                }
            }
        }

        // pretty-printing
        match status {
            StatusType::BothGood => print_log(LogLevel::Fruitful, "All preferences match!"),
            StatusType::BrewGoodOnly => print_log(
                LogLevel::Warning,
                "Homebrew apps/tools are installed but system preferences do not match. Run `cutler apply` to set.",
            ),
            StatusType::PrefsGood => print_log(LogLevel::Fruitful, "All system preferences match!"),
            StatusType::PrefsGoodOnly => print_log(
                LogLevel::Warning,
                "System preferences match but some Homebrew apps/tools are extra/missing. Run the `cutler brew` command group to sync/install.",
            ),
            StatusType::NoneGood => print_log(
                LogLevel::Warning,
                "System preferences and Homebrew apps/tools are diverged. Run `cutler apply` to apply the preferences and the `cutler brew` command group to backup/sync.",
            ),
        };

        Ok(())
    }
}
