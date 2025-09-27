// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use reqwest::Client;
use tokio::fs;
use tokio::sync::OnceCell;
use toml::Table;

use crate::util::logging::{LogLevel, print_log};

/// Manages fetching and storing the remote config.
#[derive(Debug, Clone)]
pub struct RemoteConfigManager {
    pub url: String,
    pub autosync: bool,
    config: OnceCell<Table>,
}

impl RemoteConfigManager {
    /// Create a new RemoteConfigManager with a URL.
    pub fn new(url: String) -> Self {
        Self {
            url,
            autosync: false,
            config: OnceCell::const_new(),
        }
    }

    /// Parse the [remote] section from a TOML config and create a manager.
    pub fn from_toml(config: &Table) -> Option<Self> {
        let tbl = config.get("remote")?.as_table()?;
        let url = tbl.get("url")?.as_str()?.to_string();
        let autosync = tbl
            .get("autosync")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Some(Self {
            url,
            autosync,
            config: OnceCell::const_new(),
        })
    }

    /// Fetch the remote config file as TOML, only once per instance.
    pub async fn fetch(&self) -> Result<()> {
        self.config
            .get_or_try_init(|| async {
                print_log(
                    LogLevel::Info,
                    &format!("Fetching remote config from {}", self.url),
                );
                let client = Client::builder()
                    .user_agent("cutler-remote-config")
                    .build()?;
                let resp =
                    client.get(&self.url).send().await.with_context(|| {
                        format!("Failed to fetch remote config from {}", self.url)
                    })?;

                if !resp.status().is_success() {
                    bail!("Failed to fetch remote config: HTTP {}", resp.status());
                }

                let text = resp.text().await?;
                let parsed = text
                    .parse::<Table>()
                    .with_context(|| format!("Invalid TOML config fetched from {}", self.url))?;

                Ok(parsed)
            })
            .await?;
        Ok(())
    }

    /// Save the fetched remote config to the given path.
    pub async fn save(&self, config_path: &PathBuf) -> Result<()> {
        let config = self
            .config
            .get()
            .ok_or_else(|| anyhow::anyhow!("Remote config not fetched yet"))?;
        let toml_string =
            toml::to_string(config).with_context(|| "Failed to serialize remote config to TOML")?;

        fs::create_dir_all(config_path.parent().unwrap()).await?;
        fs::write(config_path, toml_string).await?;

        print_log(
            LogLevel::Info,
            "Successfully saved remote config to destination.",
        );
        Ok(())
    }

    /// Get a reference to the fetched remote config, if available.
    pub fn get(&self) -> Option<&Table> {
        self.config.get()
    }
}
