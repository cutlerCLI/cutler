use crate::{
    brew::{
        types::BrewDiff,
        utils::{compare_brew_state, is_brew_installed},
    },
    commands::Runnable,
    config::loader::{get_config_path, load_config},
    domains::{collect, effective, read_current},
    util::{
        convert::normalize,
        logging::{BOLD, GREEN, LogLevel, RED, RESET, print_log},
    },
};
use anyhow::{Result, bail};
use async_trait::async_trait;
use clap::Args;

#[derive(Args, Debug)]
pub struct StatusCmd {
    // Disable Homebrew state check.
    #[arg(long)]
    pub no_brew: bool,
}

#[async_trait]
impl Runnable for StatusCmd {
    async fn run(&self) -> Result<()> {
        let config_path = get_config_path();
        if !config_path.exists() {
            bail!("No config file found. Please run `cutler init` first, or create a config file.");
        }

        let toml = load_config(&config_path, false).await?;
        let domains = collect(&toml)?;

        // flatten all settings into a list
        let entries: Vec<(String, String, toml::Value)> = domains
            .into_iter()
            .flat_map(|(domain, table)| {
                table
                    .into_iter()
                    .map(move |(key, value)| (domain.clone(), key.clone(), value.clone()))
            })
            .collect();

        // collect results
        let mut outcomes = Vec::with_capacity(entries.len());
        for (domain, key, value) in entries.iter() {
            let (eff_dom, eff_key) = effective(domain, key);

            let current = read_current(&eff_dom, &eff_key)
                .await
                .unwrap_or_else(|| "Not set".into());
            let desired = normalize(value);
            let is_diff = current != desired;

            outcomes.push((eff_dom, eff_key, desired, current, is_diff));
        }

        let mut any_diff = false;
        for (eff_dom, eff_key, desired, current, is_diff) in outcomes {
            if is_diff {
                any_diff = true;
                print_log(
                    LogLevel::Warning,
                    &format!(
                        "{BOLD}{eff_dom} | {eff_key}: should be {desired} (currently {RED}{current}{RESET}){RESET}",
                    ),
                );
            } else {
                print_log(
                    LogLevel::Info,
                    &format!("{GREEN}{eff_dom} | {eff_key}: {current} (matches desired){RESET}"),
                );
            }
        }

        if !any_diff {
            print_log(
                LogLevel::Fruitful,
                "All preferences already match your configuration.",
            );
        } else {
            print_log(
                LogLevel::Warning,
                "Run `cutler apply` to apply these changes from your config.\n",
            );
        }

        // brew status reporting
        if !self.no_brew {
            if let Some(brew_val) = toml.get("brew").and_then(|v| v.as_table()) {
                print_log(LogLevel::Info, "Homebrew status:");

                // ensure homebrew is installed (skip if not)
                if is_brew_installed().await {
                    print_log(
                        LogLevel::Warning,
                        &format!("Homebrew not available in PATH, skipping status check for it."),
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
                                    &format!("Formulae missing: {}", missing_formulae.join(", ")),
                                );
                            }
                            if !extra_formulae.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!(
                                        "Extra installed formulae: {}",
                                        extra_formulae.join(", ")
                                    ),
                                );
                            }
                            if !missing_casks.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!("Casks missing: {}", missing_casks.join(", ")),
                                );
                            }
                            if !extra_casks.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!("Extra installed casks: {}", extra_casks.join(", ")),
                                );
                            }
                            if !missing_taps.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!("Taps missing: {}", missing_taps.join(", ")),
                                );
                            }
                            if !extra_taps.is_empty() {
                                any_brew_diff = true;
                                print_log(
                                    LogLevel::Warning,
                                    &format!("Extra tapped: {}", extra_taps.join(", ")),
                                );
                            }
                            if !any_brew_diff {
                                print_log(
                                    LogLevel::Fruitful,
                                    "All Homebrew formulae/casks match config.",
                                );
                            } else {
                                print_log(
                                    LogLevel::Warning,
                                    "Use cutler's brew commands to sync/install these if needed.\n",
                                )
                            }
                        }
                        Err(e) => {
                            print_log(
                                LogLevel::Warning,
                                &format!("Could not check Homebrew status: {e}"),
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
