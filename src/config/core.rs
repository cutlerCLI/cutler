// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::PathBuf};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use tokio::fs;
use toml::Value;

use crate::config::path::get_config_path;

/// Struct representing a cutler configuration.
///
/// This is a fully serde-compatible struct primarily meant to be used within cutler's source code
/// to pass around information related to the config file.
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub lock: Option<bool>,
    pub set: Option<HashMap<String, HashMap<String, Value>>>,
    pub vars: Option<HashMap<String, String>>,
    pub command: Option<HashMap<String, Command>>,
    pub brew: Option<Brew>,
    pub mas: Option<Mas>,
    pub remote: Option<Remote>,
    #[serde(skip)]
    pub path: PathBuf,
}

/// Represents the [remote] table.
#[derive(Deserialize, PartialEq, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Remote {
    pub url: String,
    pub autosync: Option<bool>,
}

/// Represents [command.***] tables.
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub run: String,
    pub ensure_first: Option<bool>,
    pub required: Option<Vec<String>>,
    pub flag: Option<bool>,
    pub sudo: Option<bool>,
}

/// Represents the [mas] table.
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Mas {
    pub ids: HashMap<String, String>,
}

/// Represents the [brew] table.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Brew {
    pub formulae: Option<Vec<String>>,
    pub casks: Option<Vec<String>>,
    pub taps: Option<Vec<String>>,
    pub no_deps: Option<bool>,
}

impl Config {
    /// If the configuration file can be loaded.
    /// Since this is an independent function, it is generally encouraged over manually
    /// checking the existence of the path derived from `get_config_path()`.
    pub async fn is_loadable() -> bool {
        if let Ok(path) = get_config_path().await {
            fs::try_exists(path).await.unwrap_or_default()
        } else {
            false
        }
    }

    /// Loads the configuration. Errors out if the configuration is not loadable
    /// (decided by `Self::is_loadable()`).
    pub async fn load(not_if_locked: bool) -> Result<Self> {
        if Self::is_loadable().await {
            let config_path = get_config_path().await.unwrap();
            let data = fs::read_to_string(&config_path).await?;
            let mut config: Config = toml::from_str(&data)?;

            if config.lock.unwrap_or_default() && not_if_locked {
                bail!("Config is locked. Run `cutler unlock` to unlock.")
            }

            config.path = config_path;
            Ok(config)
        } else {
            bail!("Config path could not be decided, so cannot load.")
        }
    }

    /// Creates a new `Config` instance.
    /// Note that the path field is pre-initialized with `get_config_path()`,
    /// which can also be an empty `PathBuf`.
    pub async fn new() -> Self {
        Config {
            lock: None,
            set: None,
            vars: None,
            command: None,
            brew: None,
            remote: None,
            mas: None,
            path: get_config_path().await.unwrap_or_default(),
        }
    }

    /// Saves the configuration instance onto disk.
    /// If the parent directories do not exist, they are also created in the process.
    pub async fn save(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).await?;
        }

        let data = toml::to_string_pretty(self)?;
        fs::write(&self.path, data).await?;

        Ok(())
    }
}
