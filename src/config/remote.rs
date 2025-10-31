// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result, bail};
use reqwest::Client;
use tokio::fs;
use tokio::sync::OnceCell;

use crate::config::core::Config;
use crate::config::path::get_config_path;
use crate::log;
use crate::util::logging::LogLevel;

/// Manages fetching and storing the remote config.
#[derive(Debug, Clone)]
pub struct RemoteConfigManager {
    url: String,
    config: OnceCell<String>,
}

impl RemoteConfigManager {
    /// Create a new RemoteConfigManager with a Remote struct.
    pub fn new(url: String) -> Self {
        Self {
            url,
            config: OnceCell::const_new(),
        }
    }

    /// Fetch the remote config file as TOML, only once per instance.
    pub async fn fetch(&self) -> Result<()> {
        self.config
            .get_or_try_init(|| async {
                log!(LogLevel::Info, "Fetching remote config from {}", self.url,);
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

                toml::from_str::<Config>(&text)
                    .with_context(|| format!("Invalid TOML config fetched from {}", self.url))?;

                Ok(text)
            })
            .await?;
        Ok(())
    }

    /// Save the fetched remote config to the given path.
    pub async fn save(&self) -> Result<()> {
        let config = self.get()?;
        let config_path = get_config_path().await?;

        fs::create_dir_all(config_path.parent().unwrap()).await?;
        fs::write(config_path, config).await?;
        log!(
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
