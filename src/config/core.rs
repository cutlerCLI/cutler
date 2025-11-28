// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tokio::fs;
use toml::Value;
use toml_edit::DocumentMut;

/// Struct representing a loaded cutler configuration.
///
/// This is a fully serde-compatible struct primarily meant to be used within cutler's source code
/// to pass around information related to the config file.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct LoadedConfig {
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
#[derive(Deserialize, PartialEq, Eq, Default, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Remote {
    pub url: String,
    pub autosync: Option<bool>,
}

/// Represents [command.***] tables.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub run: String,
    pub ensure_first: Option<bool>,
    pub required: Option<Vec<String>>,
    pub flag: Option<bool>,
    pub sudo: Option<bool>,
}

/// Represents the [mas] table.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Mas {
    pub ids: Vec<String>,
}

/// Represents the [brew] table.
#[derive(Deserialize, PartialEq, Eq, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Brew {
    pub formulae: Option<Vec<String>>,
    pub casks: Option<Vec<String>>,
    pub taps: Option<Vec<String>>,
    pub no_deps: Option<bool>,
}

/// Represents an unloaded cutler configuration.
///
/// This must be loaded with .load() to return a LoadedConfig, or .load_as_mut() to return a toml_edit::DocumentMut.
pub struct Config {
    pub path: PathBuf,
}

impl Config {
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn is_loadable(&self) -> bool {
        !self.path.as_os_str().is_empty() && self.path.try_exists().unwrap_or(false)
    }

    /// Loads the configuration. Errors out if the configuration is not loadable
    /// (decided by `.is_loadable()`).
    pub async fn load(&self, not_if_locked: bool) -> Result<LoadedConfig> {
        if self.is_loadable() {
            let data = fs::read_to_string(&self.path).await?;

            let mut config: LoadedConfig =
                toml::from_str(&data).context("Failed to parse config data from valid TOML.")?;

            if config.lock.unwrap_or_default() && not_if_locked {
                bail!("Config is locked. Run `cutler unlock` to unlock.")
            }

            config.path = self.path.to_owned();
            Ok(config)
        } else {
            bail!("Config path does not exist!")
        }
    }

    /// Loads config as mutable `DocumentMut`. Useful for in-place editing of values.
    pub async fn load_as_mut(&self, not_if_locked: bool) -> Result<DocumentMut> {
        if self.is_loadable() {
            let data = fs::read_to_string(&self.path).await?;
            let config: LoadedConfig =
                toml::from_str(&data).context("Failed to parse config data from valid TOML.")?;

            if config.lock.unwrap_or_default() && not_if_locked {
                bail!("Config is locked. Run `cutler unlock` to unlock.")
            }

            let doc = data.parse::<DocumentMut>()?;

            Ok(doc)
        } else {
            bail!("Config path does not exist!")
        }
    }
}

/// Trait for implementing core Config struct methods for other types.
///
/// Purely convenience.
pub trait ConfigCoreMethods {
    fn save(&self, path: &Path) -> impl Future<Output = Result<()>>;
}

impl ConfigCoreMethods for DocumentMut {
    /// Saves the document into the conventional configuration path decided during runtime.
    async fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).await?;
        }

        let data = self.to_string();
        fs::write(path, data).await?;

        Ok(())
    }
}
