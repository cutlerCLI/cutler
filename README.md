<img src="assets/logo.png" width="180px" align="right">

# 🍎 cutler

#### Setup automation for your Mac

[![Crates.io Downloads](https://img.shields.io/crates/d/cutler?style=social&logo=Rust)](https://crates.io/crates/cutler)
[![Rust Tests](https://github.com/cutlerCLI/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/cutlerCLI/cutler/actions/workflows/tests.yml)

Pre-built binaries support **macOS Big Sur (11.0) or later** versions.

> [!WARNING]
> Expect breaking changes before v1.0.0 release.

## Quick Start

```bash
# Self-installing script
# See below sections for other methods.
curl -fsSL https://cutlercli.github.io/scripts/install.sh | /bin/bash

# Initialize a configuration file.
# Basic template includes preferences, Homebrew and external commands.
cutler init

# Modify using your preferred editor.
nano ~/.config/cutler/config.toml

# Apply your preferences.
cutler apply
```

## What is cutler?

cutler simplifies the macOS setup pipeline by:

- providing a declarative way to set system settings, without touching the Settings app,
- wrapping around package managers for easy apps/tooling installation, and
- by providing first-class support for external commands to ensure easy extensibility.

All of this happens with a single `cutler.toml` file in your config directory.

## How it works

For backend functionality over system preferences, cutler wraps around the preferences APIs exposed by macOS. This allows for direct and instant feedback by just restarting the corresponding services related to the preference.

For wrapping around tooling, cutler includes mature implementations for extracting the data from the selected package managers.
Sensitive actions (e.g. installing) are currently outsourced to remain as close to the original functionality as possible.

cutler is still in development and changes in functionality may be common during this stage.

## Useful Links

- [Resources](#resources)
- [Installation](#installation)
- [Contributing](#contributing)
- [License](#license)

## Resources

- [**Complete Documentation (Cookbook)**](https://cutlercli.github.io/cookbook)
- [macOS defaults list](https://macos-defaults.com) (useful if you're starting out with declaring bare-metal system preferences and have not previously used `defaults`)

## Installation

### Self-install (recommended)

```bash
curl -fsSL https://cutlercli.github.io/scripts/install.sh | /bin/bash
```

### Using Homebrew

```bash
brew install hitblast/tap/cutler
```

### Using cargo

```bash
cargo install cutler
```

### Using mise

```bash
mise use -g cargo:cutler
```

## Contributing

View the [Contribution Guidelines](https://cutlercli.github.io/cookbook/guidelines/contributing.html) to learn more about contributing to cutler. It also contains resources such as code snippets to make your contribution workflow easier.

## License

This project is permissively licensed and free forever. See the license files mentioned below for the details:

- Apache Software License 2.0 [(LICENSE-APACHE)](https://github.com/cutlerCLI/cutler/blob/master/LICENSE-APACHE)
- MIT License [(LICENSE-MIT)](https://github.com/cutlerCLI/cutler/blob/master/LICENSE-MIT)
