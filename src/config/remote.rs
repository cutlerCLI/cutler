use anyhow::{Context, Result, bail};
use reqwest::Client;
use toml::Value;

use crate::util::logging::{LogLevel, print_log};

/// Represents the [remote] config section
#[derive(Debug, Clone)]
pub struct RemoteConfig {
    pub url: String,
    pub autosync: bool,
}

impl RemoteConfig {
    pub fn from_toml(config: &Value) -> Option<Self> {
        let tbl = config.get("remote")?.as_table()?;
        let url = tbl.get("url")?.as_str()?.to_string();

        let autosync = tbl
            .get("autosync")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Some(Self { url, autosync })
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
            bail!("Failed to fetch remote config: HTTP {}", resp.status());
        }

        let text = resp.text().await?;
        let parsed = text
            .parse::<Value>()
            .with_context(|| format!("Invalid TOML config fetched from {}", self.url))?;

        Ok(parsed)
    }
}

/// Merge remote config into local config, preserving [remote] if not present in remote.
pub fn merge_remote_config(local: &toml::Value, remote: &toml::Value) -> toml::Value {
    let empty_map = toml::map::Map::new();
    let remote_table = remote.as_table().unwrap_or(&empty_map);
    let local_table = local.as_table().unwrap_or(&empty_map);

    let mut merged_table = remote_table.clone();

    if !remote_table.contains_key("remote") {
        if let Some(local_remote) = local_table.get("remote") {
            merged_table.insert("remote".to_string(), local_remote.clone());
        }
    }
    toml::Value::Table(merged_table)
}
