// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Display;

/// Represents the type of software to list in Homebrew.
#[derive(PartialEq)]
pub enum BrewListType {
    /// Lists casks (inside caskroom).
    Cask,
    /// Lists formulae (inside cellar).
    Formula,
    /// Lists only the dependencies.
    Dependency,
    /// Lists taps.
    Tap,
}

impl Display for BrewListType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flag = match self {
            BrewListType::Cask => "--cask",
            BrewListType::Formula => "--formula",
            BrewListType::Dependency => "--installed-as-dependency",
            BrewListType::Tap => "tap",
        };
        write!(f, "{}", flag)
    }
}

/// Struct representing the diff between config and installed Homebrew state.
#[derive(Debug, Default)]
pub struct BrewDiff {
    pub missing_formulae: Vec<String>,
    pub extra_formulae: Vec<String>,
    pub missing_casks: Vec<String>,
    pub extra_casks: Vec<String>,
    pub missing_taps: Vec<String>,
    pub extra_taps: Vec<String>,
}
