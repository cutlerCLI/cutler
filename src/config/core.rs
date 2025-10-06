// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::PathBuf};

use anyhow::{Result, bail};
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
    pub path: PathBuf,
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
        if let Ok(path) = get_config_path().await {
            fs::try_exists(path).await.unwrap_or_default()
        } else {
            false
        }
    }

    pub async fn load() -> Result<Self> {
        if Self::is_loadable().await {
            let config_path = get_config_path().await.unwrap();
            let data = fs::read_to_string(&config_path).await?;
            let mut config: Config = toml::from_str(&data)?;

            config.path = config_path;
            Ok(config)
        } else {
            bail!("Config path could not be decided, so cannot load.")
        }
    }

    pub async fn new() -> Self {
        Config {
            lock: None,
            set: None,
            vars: None,
            command: None,
            brew: None,
            remote: None,
            path: get_config_path().await.unwrap_or_default(),
        }
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).await?;
        }

        let data = toml::to_string(self)?;
        fs::write(&self.path, data).await?;

        Ok(())
    }
}
