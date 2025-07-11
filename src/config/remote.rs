use anyhow::{Context, Result, bail};
use reqwest::Client;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::OnceCell;
use toml::Value;

use crate::{
    config::loader::{get_config_path, load_config},
    util::logging::{LogLevel, print_log},
};

/// Global OnceCell holding the fetched remote config.
/// This is accessible from outside the struct.
pub static REMOTE_CONFIG: OnceCell<Value> = OnceCell::const_new();

/// Merge remote config into local config, preserving [remote] if not present in remote.
fn merge_remote_config(local: &toml::Value, remote: &toml::Value) -> toml::Value {
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

/// Represents the [remote] config section.
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

    /// Fetch the remote config file as TOML, only once per process lifetime.
    pub async fn fetch(&self) -> Result<()> {
        REMOTE_CONFIG
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
                    .parse::<Value>()
                    .with_context(|| format!("Invalid TOML config fetched from {}", self.url))?;

                Ok(parsed)
            })
            .await?;
        Ok(())
    }

    /// Save the fetched remote config to the given path.
    pub async fn save(&self) -> Result<()> {
        let path = get_config_path().await;
        let config = REMOTE_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Remote config not fetched yet"))?;

        let toml_string =
            toml::to_string(config).with_context(|| "Failed to serialize remote config to TOML")?;

        let mut file = fs::File::create(path).await?;
        file.write_all(toml_string.as_bytes()).await?;

        print_log(
            LogLevel::Info,
            "Successfully saved remote config to destination.",
        );
        Ok(())
    }

    /// Merge the fetched remote config into the local config, preserving [remote] if not present in remote,
    /// and save the merged config to disk.
    pub async fn save_merge_local(&self) -> Result<()> {
        let local = load_config(false).await?;
        let remote = REMOTE_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Remote config not fetched yet"))?;
        let merged = merge_remote_config(&local, remote);

        let path = get_config_path().await;
        let toml_string = toml::to_string(&merged)
            .with_context(|| "Failed to serialize merged config to TOML")?;

        let mut file = fs::File::create(path).await?;
        file.write_all(toml_string.as_bytes()).await?;

        print_log(
            LogLevel::Info,
            "Successfully saved merged config to destination.",
        );
        Ok(())
    }
}
