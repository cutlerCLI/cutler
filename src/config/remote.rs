use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use toml::Value;

use crate::util::logging::{LogLevel, print_log};

/// Represents the [remote] config section
#[derive(Debug, Clone)]
pub struct RemoteConfig {
    pub url: String,
    pub update_on_cmd: bool,
}

impl RemoteConfig {
    pub fn from_toml(config: &Value) -> Option<Self> {
        let tbl = config.get("remote")?.as_table()?;
        let url = tbl.get("url")?.as_str()?.to_string();

        let update_on_cmd = tbl
            .get("update_on_cmd")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Some(Self { url, update_on_cmd })
    }

    /// Fetch the remote config file as TOML.
    pub async fn fetch(&self) -> Result<Value> {
        print_log(
            LogLevel::Info,
            &format!("Fetching remote config from {}", self.url),
        );

        let client = Client::builder()
            .user_agent("cutler-remote-config")
            .build()?;

        let resp = client
            .get(&self.url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch remote config from {}", self.url))?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch remote config: HTTP {}",
                resp.status()
            ));
        }

        let text = resp.text().await?;
        let remote_config: Value = text
            .parse::<Value>()
            .with_context(|| format!("Failed to parse remote config as TOML from {}", self.url))?;

        Ok(remote_config)
    }
}
