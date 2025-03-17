<div align="center">

<img src="assets/logo.png" width="200px">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

Declarative macOS settings management at your fingertips, with speed. <br>

[![Release Builds](https://github.com/hitblast/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/release.yml)

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
- [Installation Methods](#other-installation-methods)
- [Usage](#usage)
- [Resources](#resources)
- [Notable things](#notable-things)
- [Contributing](#contributing)
- [License](#license)

## Overview

If you use macOS, you might be familiar with changing settings using the
built-in System Settings app or the `defaults` command in the terminal. Both
methods can be tedious—and the terminal option usually involves manual tweaks.
That’s where `cutler` makes things simpler!

`cutler` is a straightforward command-line tool that lets you specify your macOS
preferences in an easy-to-read TOML file. It wraps the `defaults` command so you
can quickly apply or undo settings when needed.

Check out the [Usage](#usage) section for more details.

## Installation Methods

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

`cutler` looks for your configuration in a file named `config.toml`, which can be located in one of these spots:

- `$XDG_CONFIG_HOME/cutler/config.toml` or,
- `~/.config/cutler/config.toml`

It respects your `$XDG_CONFIG_HOME` setting, so you don't have to worry about
path issues. Just place your `config.toml` file in one of these locations and
you're set.

Here’s a basic example of a TOML configuration:

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

If you run `cutler apply` for the first time without an existing configuration
file, it will generate a sample config for you. You can also check out the
complete example in
[examples/cutler.toml](https://github.com/hitblast/cutler/blob/main/examples/cutler.toml).

Once your configuration file is ready, apply your settings by running:

```bash
cutler apply
```

> [!NOTE]
> After `cutler` updates the defaults, it restarts the necessary system services on your Mac so that the changes take effect. Some services might even require a full reboot to fully apply the new settings.

Sometimes you may want to check that your settings have been correctly
applied—or if they have been changed. To do that, run:

```bash
cutler status
```

To revert all modifications, run:

```bash
cutler unapply
```

And if you decide to completely revert everything to factory defaults, run:

```bash
cutler delete
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

## Contributing

This is a personal project aimed at making the task of setting up a Mac more
straightforward. Contributions are always welcome! Feel free to help out by
[creating a pull request]() or [submitting an issue]().

## License

This project is licensed under the [MIT License](https://github.com/hitblast/cutler/blob/main/LICENSE).
