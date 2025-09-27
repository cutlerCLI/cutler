// SPDX-License-Identifier: Apache-2.0

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
