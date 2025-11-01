<img src="assets/logo.png" width="180px" align="right">

# cutler

#### Setup automation for your Mac

[![Crates.io Downloads](https://img.shields.io/crates/d/cutler?style=social&logo=Rust)](https://crates.io/crates/cutler)
[![Rust Tests](https://github.com/cutlerCLI/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/cutlerCLI/cutler/actions/workflows/tests.yml)

Turn your macOS setup workflow into a one-command procedure. System preferences, apps, tooling, you name it - cutler can automate it with a single file!

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

# Apply your preferences
cutler apply
```

## Useful Links

- [Resources](#resources)
- [Installation](#installation)
- [Contributing](#contributing)
- [License](#license)

## Resources

- [**Complete Documentation (Cookbook)**](https://cutlercli.github.io/cookbook)

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

This project is dual-licensed:

- **Open Source License:** [GNU General Public License v3.0 or later (GPLv3)](https://github.com/cutlerCLI/cutler/blob/master/LICENSE.md)
- **Commercial License:** See [COMMERCIAL_LICENSE.md](https://github.com/cutlerCLI/cutler/blob/master/COMMERCIAL_LICENSE.md) for terms

You may choose either license for your use case.
For commercial licensing or support, contact **Anindya Shiddhartha** at [hitblastlive@gmail.com](mailto:hitblastlive@gmail.com).
