// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result, bail};
use reqwest::Client;
use tokio::fs;
use tokio::sync::OnceCell;

use crate::config::core::{Config, Remote};
use crate::config::path::get_config_path;
use crate::util::logging::{LogLevel, print_log};

/// Manages fetching and storing the remote config.
#[derive(Debug, Clone)]
pub struct RemoteConfigManager {
    pub remote: Remote,
    config: OnceCell<String>,
}

impl RemoteConfigManager {
    /// Create a new RemoteConfigManager with a Remote struct.
    pub fn new(remote: Remote) -> Self {
        Self {
            remote,
            config: OnceCell::const_new(),
        }
    }

    /// Parse the [remote] section from a serde-based Config and create a manager.
    pub fn from_config(config: &Config) -> Option<Self> {
        config.remote.clone().map(Self::new)
    }

    /// Fetch the remote config file as TOML, only once per instance.
    pub async fn fetch(&self) -> Result<()> {
        self.config
            .get_or_try_init(|| async {
                print_log(
                    LogLevel::Info,
                    &format!("Fetching remote config from {}", self.remote.url),
                );
                let client = Client::builder()
                    .user_agent("cutler-remote-config")
                    .build()?;
                let resp = client.get(&self.remote.url).send().await.with_context(|| {
                    format!("Failed to fetch remote config from {}", self.remote.url)
                })?;

                if !resp.status().is_success() {
                    bail!("Failed to fetch remote config: HTTP {}", resp.status());
                }

                let text = resp.text().await?;

                toml::from_str::<Config>(&text).with_context(|| {
                    format!("Invalid TOML config fetched from {}", self.remote.url)
                })?;

                Ok(text)
            })
            .await?;
        Ok(())
    }

    /// Save the fetched remote config to the given path.
    pub async fn save(&self) -> Result<()> {
        let config = self.get()?;

        let config_path = get_config_path().await;
        fs::create_dir_all(config_path.parent().unwrap()).await?;
        fs::write(config_path, config).await?;

        print_log(
            LogLevel::Info,
            "Successfully saved remote config to destination.",
        );
        Ok(())
    }

    /// Get a reference to the fetched remote config, if available.
    pub fn get(&self) -> Result<&String> {
        let config = self
            .config
            .get()
            .ok_or_else(|| anyhow::anyhow!("Remote config not fetched yet"))?;

        Ok(config)
    }

    /// Get a parsed version of the output of .get() as serde-based Config.
    pub fn get_parsed(&self) -> Result<Config> {
        let config_str = self.get()?;
        let config = toml::from_str::<Config>(config_str)?;
        Ok(config)
    }
}
