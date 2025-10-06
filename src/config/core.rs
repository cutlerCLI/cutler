// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;
use toml::Value;

use crate::config::path::get_config_path;

/// Struct representing a cutler configuration
/// This is a fully serde-compatible struct primarily meant to be used within cutler's source code
/// to pass around information related to the config file.
#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub lock: Option<bool>,
    pub set: Option<HashMap<String, HashMap<String, Value>>>,
    pub vars: Option<HashMap<String, String>>,
    pub command: Option<HashMap<String, Command>>,
    pub brew: Option<Brew>,
    pub remote: Option<Remote>,
    #[serde(skip)]
    pub config_path: PathBuf,
}

/// Represents the [remote] table.
#[derive(Deserialize, PartialEq, Serialize, Clone, Debug)]
pub struct Remote {
    pub url: String,
    pub autosync: Option<bool>,
}

/// Represents [command.***] tables.
#[derive(Deserialize, Serialize, Clone)]
pub struct Command {
    pub run: String,
    pub ensure_first: Option<bool>,
    pub required: Option<Vec<String>>,
    pub flag: Option<bool>,
    pub sudo: Option<bool>,
}

/// Represents the [brew] table.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug, Default)]
pub struct Brew {
    pub formulae: Option<Vec<String>>,
    pub casks: Option<Vec<String>>,
    pub taps: Option<Vec<String>>,
    pub no_deps: Option<bool>,
}

impl Config {
    pub async fn is_loadable() -> bool {
        fs::try_exists(get_config_path().await)
            .await
            .unwrap_or_default()
    }

    pub async fn load() -> Result<Self> {
        let config_path = get_config_path().await;
        let data = fs::read_to_string(&config_path).await?;
        let mut config: Config = toml::from_str(&data)?;

        config.config_path = config_path;
        Ok(config)
    }

    pub async fn new() -> Self {
        Config {
            lock: None,
            set: None,
            vars: None,
            command: None,
            brew: None,
            remote: None,
            config_path: get_config_path().await,
        }
    }

    pub async fn save(&self) -> Result<()> {
        let data = toml::to_string(self)?;

        if let Some(dir) = self.config_path.parent() {
            fs::create_dir_all(dir).await?;
        }

        fs::write(&self.config_path, data).await?;
        Ok(())
    }
}
