# The cutler Cookbook

<div align="center">

<img src="https://raw.githubusercontent.com/hitblast/cutler/main/assets/logo.png" width="200px">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

Powerful, declarative settings management for your Mac, with speed.

[![Release Builds](https://github.com/hitblast/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/release.yml)
[![Rust Tests](https://github.com/hitblast/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/tests.yml)

</div>

> [!WARNING]
> Although cutler is solid enough for daily-driving now, expect breaking changes before the v1 release.

## Overview

cutler aims to simplify your macOS setup experience into an "almost" one-command procedure. Define your settings once, then easily apply, track, and revert changes across your system—think of it as infrastructure-as-code for your Mac.

> [!WARNING]
> This project is still under development. So, if you like it, consider starring! It's free, and it always supports me to make such projects.

## Key Features

- Manage your Mac's system preferences with just one TOML file.
- Track your Homebrew formulae/casks and back them up to restore later.
- Run external commands, both as hooks, or at your will.
- Revert back modifications easily with the snapshot mechanism.
- Do all of these things at an incredibly fast speed, thanks to Rust.