<div align="center">

<img src="assets/logo.png" width="200px">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

Powerful, declarative settings management for your Mac, with speed.

[![Release Builds](https://github.com/hitblast/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/release.yml)
[![Rust Tests](https://github.com/hitblast/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/tests.yml)

</div>

## 🍺 Installation

Install cutler using [Homebrew](https://brew.sh) by simply running:

```bash
brew install hitblast/tap/cutler
```

> [!IMPORTANT]
> The prebuilt binaries are compiled and shipped from macOS 14 on arm64.
> Intel Macs will require a manual compilation of the project.

## Table of Contents

- [Overview](#overview)
- [Other Installation Methods](#other-installation-methods)
- [Usage](#usage)
  - [Anatomy](#basic-anatomy)
  - [Defaults and External Commands](#defaults-and-external-commands)
  - [Applying Changes and Status Review](#applying-changes-and-status-review)
- [Resources](#resources)
- [Notable Things](#notable-things)
- [Contributing](#contributing)
- [License](#license)

## Overview

If you use macOS, you might be familiar with changing settings using the
built-in System Settings app or the `defaults` command in the terminal. Both
methods can be tedious—and the terminal option usually involves manual tweaks.
That’s where `cutler` makes things simpler!

`cutler` is a straightforward command-line tool that lets you specify your macOS
preferences in an easy-to-read TOML file. It wraps the `defaults` command so you
can quickly apply or undo settings when needed. In addition to managing macOS defaults,
cutler now supports executing external commands so that you don't have to write another
shell script to automate things.

Check out the [Usage](#usage) section for more details.

## Other Installation Methods

Besides using Homebrew as shown above, you can install the project in a couple of other ways:

- Using `cargo`:

```bash
cargo install cutler
```

- Using `mise`:

```bash
# NOTE: This will compile the binary manually for your system.
mise use -g cargo:cutler
```

> [!TIP]
> If none of these installation methods work for you, try checking out the latest GitHub release.
> You can also use the periodic release workflows, which have a retention period of 90 days.

## Usage

`cutler` looks for your configuration in a file named `config.toml`, checking the following locations in order:

- `$XDG_CONFIG_HOME/cutler/config.toml`
- `~/.config/cutler/config.toml`
- `~/.config/cutler.toml`
- `config.toml` in the current directory (fallback)

It respects your `$XDG_CONFIG_HOME` setting, so you don't have to worry about
path issues. Just place your `config.toml` file in one of these locations and
you're set.

To easily get started, simply type the following command to generate a prebuilt configuration:

```bash
cutler init
```

### Anatomy

Here’s a basic example of a TOML configuration for cutler:

```toml
[dock]
tilesize = 46

[menuextra.clock]
FlashDateSeparators = true
```

For more details on the different `defaults` domains and available values on
macOS, take a look at the [Resources](#resources) section. The TOML above
translates into these commands:

```bash
defaults write com.apple.dock "tilesize" -int "46"
defaults write com.apple.menuextra.clock "FlashDateSeparators"
```

You can also configure settings for `NSGlobalDomain` like this:

```toml
[NSGlobalDomain]
ApplePressAndHoldEnabled = true

[NSGlobalDomain.com.apple.mouse]
linear = true
```

`cutler` converts the above TOML into:

```bash
defaults write NSGlobalDomain "ApplePressAndHoldEnabled" -bool true
defaults write NSGlobalDomain com.apple.mouse.linear -bool true
```

> [!WARNING]
> Currently, `cutler` does not verify the integrity of domains or keys under `NSGlobalDomain`. Please review these settings manually before applying any changes.

### Defaults and External Commands

Beyond managing macOS defaults, cutler now supports an `[external]` section that allows you to run any external command after applying the defaults. This is particularly useful when you want to trigger additional scripts or commands as part of your configuration. For example:

```toml
# Define reusable variables here:
[external.variables]
common_args = ["Hello", "World"]

[external]
  [[external.command]]
  cmd = "echo"
  # If you reference a variable (for example, $common_args) and it isn’t defined
  # in the [external.variables] section, cutler will fall back and try to resolve it
  # from the environment (e.g. $PATH).
  args = ["$common_args", "$PATH"]
  sudo = false
```

This roughly translates to:

```bash
echo Hello World /usr/local/bin:/usr/bin:...
```

If you don't want to run into additional giberish, the external commands only require the `cmd` field to run, so it can be as simple as:

```toml
[external]
  [[external.command]]
  cmd = "echo"
```

### Applying Changes and Status Review

Once your configuration file is ready (including your defaults and external commands), apply your settings by running:

```bash
cutler apply
```

After `cutler` updates the defaults, it will also:
- Execute any external commands defined in the `[external]` section.
- Restart necessary system services on your Mac so that the new settings take effect.

To verify current settings against your configuration, run:

```bash
cutler status
```

To revert modifications, run:

```bash
cutler unapply
```

Now, when it comes to managing the configuration file itself, there is a `config` command which has two other subcommands:

```bash
# Shows the contents of the configuration file.
cutler config show

# Unapplies and deletes the configuration file.
cutler config delete
```

You can add `--verbose` for more detail on what happens behind the scenes. For
additional information about all available commands, run:

```bash
cutler help
```

## Notable Things

When you run `cutler apply`, a snapshot file named `.cutler_snapshot` is created
in your home directory. This file records your configuration state and lets you
revert to a previous setup if needed. It’s important not to overwrite or delete
this file manually, as it is essential for maintaining the integrity of your
configuration.

## Resources

Finding the ideal set of macOS defaults can be challenging. Visit the [macOS
defaults website](https://macos-defaults.com/) for a comprehensive list of
available settings.

Sample configuration files are preincluded with this repository for you to have a look
at and get hold of the tool quickly:

- [examples/basic.toml](examples/basic.toml) (for minimal usage)
- [examples/advanced.toml](examples/advanced.toml)

## Contributing

This is a personal project aimed at making the task of setting up a Mac more
straightforward. Contributions are always welcome! Feel free to help out by
[creating a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request) or [submitting an issue](https://github.com/hitblast/cutler/issues).

## License

This project is licensed under the [MIT License](https://github.com/hitblast/cutler/blob/main/LICENSE).
