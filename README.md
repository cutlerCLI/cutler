<img src="assets/logo.png" width="200px" align="right">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

[![Release Builds](https://github.com/hitblast/trimsec/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/trimsec/actions/workflows/release.yml)

Declarative macOS defaults management at your fingertips, with speed.

> [!WARNING]
> This project is still under active development. Some of the
> written functionality here might be missing. Please wait for the initial
> release.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Overview

cutler is a command-line tool that allows you to manage macOS defaults with a
simple TOML file. It is made using Rust to facilitate tiny binary size for its
functionality, and obviously, speed.

Many of us tend to use the `defaults` command-line tool to
declare specific settings on our Macs. However, this boils down to unnecessary
scripting and might also increase the chances of accidentally harming your
default configurations.

## Installation

- Using `cargo`:

```bash
cargo install cutler
```

- Using `mise`:

```bash
mise use -g cargo
```

- Using Homebrew:

```bash
# still a work-in-progress
```

## Usage

The command-line interface respects `$XDG_CONFIG_HOME` and depending on your
macOS settings, `cutler` will read the `config.toml` for your written defaults
from this path:

- `$XDG_CONFIG_HOME/cutler/config.toml` or,
- `~/.config/cutler/config.toml`

The overall structure for cutler's configuration should be as follows:

```toml
[dock]
tilesize = 46

[menuextra.clock]
FlashDateSeparators = true
```

Here, the following TOML code translates to the following command being executed:

```bash
defaults write com.apple.dock "tilesize" -int "46"
defaults write com.apple.menuextra.clock "FlashDateSeparators"
```

The interface is also type-safe and uses generic TOML parsing to ensure that the
types are designated properly. So, to apply this configuration, simply run:

```bash
cutler apply
```

Please note that `cutler apply` also generates a default configuration file for
you to get started with if no file is found in the designated directories.
SImply accept the prompt and you're good to go.

Now, to unapply the configuration, simply run:

```bash
cutler unapply
```

If you'd like to remove the file entirely, you can run the following command:
(generally not recommended since currently applied settings could become untrackable)

```bash
cutler delete
```

## Contributing

This is one of my hobby projects that I've made to solve one of my most common
problems - repeatedly applying the same settings on my MacBook. So, pull
requests are always welcome! Feel free to submit your changes or improvements by
[creating a pull request]() or by [submitting an issue]().

## License

This project has been licensed under the [MIT License](LICENSE).
