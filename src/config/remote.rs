use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use reqwest::Client;
use tokio::fs;
use tokio::sync::OnceCell;
use toml::Table;

use crate::util::logging::{LogLevel, print_log};

/// Global OnceCell holding the fetched remote config.
/// This is accessible from outside the struct.
pub static REMOTE_CONFIG: OnceCell<Table> = OnceCell::const_new();

/// Represents the [remote] config section.
#[derive(Debug, Clone)]
pub struct RemoteConfig {
    pub url: String,
    pub autosync: bool,
}

impl RemoteConfig {
    pub fn from_toml(config: &Table) -> Option<Self> {
        let tbl = config.get("remote")?.as_table()?;
        let url = tbl.get("url")?.as_str()?.to_string();

        let autosync = tbl
            .get("autosync")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Some(Self { url, autosync })
    }
}

/// Fetch the remote config file as TOML, only once per process lifetime.
pub async fn fetch_remote_config(url: String) -> Result<()> {
    REMOTE_CONFIG
        .get_or_try_init(|| async {
            print_log(
                LogLevel::Info,
                &format!("Fetching remote config from {url}"),
            );

            let client = Client::builder()
                .user_agent("cutler-remote-config")
                .build()?;
            let resp = client
                .get(&url)
                .send()
                .await
                .with_context(|| format!("Failed to fetch remote config from {url}"))?;

            if !resp.status().is_success() {
                bail!("Failed to fetch remote config: HTTP {}", resp.status());
            }

            let text = resp.text().await?;
            let parsed = text
                .parse::<Table>()
                .with_context(|| format!("Invalid TOML config fetched from {url}"))?;

            Ok(parsed)
        })
        .await?;
    Ok(())
}

/// Save the fetched remote config to the given path.
pub async fn save_remote_config(config_path: &PathBuf) -> Result<()> {
    let config = REMOTE_CONFIG
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
