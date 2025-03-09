<img src="https://github.com/hitblast/cutler/blob/v0.1.0/assets/logo.png" width="200px" align="right">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

[![Release Builds](https://github.com/hitblast/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/release.yml)

Declarative macOS settings management at your fingertips, with speed. <br>

```bash
# Install via Homebrew.
brew install hitblast/tap/cutler
```

> [!IMPORTANT]
> The prebuilt binaries are compiled and shipped from macOS 14 on arm64.
> They’re designed to work on Macs with the same architecture.

## Table of Contents

- [Overview](#overview)
- [Installation Methods](installation-methods)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Overview

Most of us, who use macOS, either configure it using the built-in System
Settings app, or the `defaults` command-line tool. Both of these options are
tedious and the latter requires manual tinkering with the settings inside a
terminal. `cutler` is a project which solves just that!

`cutler` is a simple, command-line tool that lets you define your macOS system
preferences in a TOML file. It wraps the `defaults` command, giving you an easy
way to apply or reverse settings on the go.

Check out the [Usage](#usage) section for more details.

## Installation Methods

In addition to using Homebrew (as shown above), you can also install `cutler`
via the `cargo` package manager or [mise](https://mise.jdx.dev).

- Using `cargo`:

```bash
cargo install cutler
```

- Using `mise`:

```bash
mise use -g cargo:cutler
```

> [!TIP]
> If you can’t find an installation method that works for you, try checking out the latest GitHub release.
> Alternatively, you can opt for the periodic release workflows, which keep releases for 90 days.

## Usage

`cutler` reads your configuration from a `config.toml` file, which can live in one of these locations:

- `$XDG_CONFIG_HOME/cutler/config.toml` or,
- `~/.config/cutler/config.toml`

It even respects `$XDG_CONFIG_HOME` so you don't have to worry about path
issues. Just drop your `config.toml` file in one of these spots and you're ready
to go.

Here’s what a basic TOML configuration looks like:

```toml
[dock]
tilesize = 46

[menuextra.clock]
FlashDateSeparators = true
```

For more details on the different `defaults` domains and values available for
macOS, check out the [Resources](#resources) section. The TOML above effectively
translates to these commands:

```bash
defaults write com.apple.dock "tilesize" -int "46"
defaults write com.apple.menuextra.clock "FlashDateSeparators"
```

You can also set options for `NSGlobalDomain` like this:

```toml
[NSGlobalDomain]
ApplePressAndHoldEnabled = true

[NSGlobalDomain.com.apple.mouse]
linear = true
```

`cutler` will translate the following TOML to:

```bash
defaults write NSGlobalDomain "ApplePressAndHoldEnabled" -bool true
defaults write NSGlobalDomain com.apple.mouse.linear -bool true
```

Note that if you run `cutler apply` for the first time without a configuration
file, it will generate a sample config for you. You can also take a look at
[examples/cutler.toml](https://github.com/hitblast/cutler/blob/main/examples/cutler.toml)
for a full example.

Once you’ve set up your file, apply your settings with:

```bash
cutler apply
```

> [!NOTE]
> After `cutler` updates the defaults, it restarts the relevant system services on your Mac so that the changes take effect.
> Some services might even require a full reboot to get fully applied.

To unapply the settings, run:

```bash
cutler unapply
```

And if you want to completely remove your configuration file (note: this might
make it harder to keep track of your settings), run:

```bash
cutler delete
```

You can use `--verbose` to see more details about the behind-the-scenes command
execution. More information about all of the commands can be found by running
`cutler help`.

## Resources

Finding the perfect set of defaults can be a bit of a hassle. Check out [the
"macOS defaults" website](https://macos-defaults.com/) for a comprehensive list
of settings.

## Contributing

This is a passion project of mine to simplify the repetitive task of setting up
my MacBook. Pull requests are always welcome! Feel free to contribute by
[creating a pull request]() or [submitting an issue]().

## License

This project is licensed under the [MIT License](https://github.com/hitblast/cutler/blob/main/LICENSE).
