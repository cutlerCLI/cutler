<div align="center">

<img src="assets/logo.png" width="200px">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

Powerful, declarative settings management for your Mac, with speed.

[![Release Builds](https://github.com/hitblast/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/release.yml)
[![Rust Tests](https://github.com/hitblast/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/tests.yml)

</div>

> [!WARNING]
> Although cutler is solid enough for daily-driving now, expect breaking changes before the v1 release.

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Installation](#installation)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Shell Completions](#shell-completions)
- [Resources](#resources)
- [Contributing](#contributing)
- [License](#license)

## Overview

cutler simplifies macOS configuration by letting you manage system settings through a single TOML file instead of clicking through System Settings or typing complex defaults commands in the terminal.

Define your settings once, then easily apply, track, and revert changes across your system—think of it as infrastructure-as-code for your Mac.

Check out the [Usage](#usage) section for more details.

## Key Features

- Manage the system `defaults` entries using just one TOML file.
- Run external commands at will with ease.
- Revert back modifications easily with the snapshot mechanism.
- Made using [Rust](https://rust-lang.org/) for thread-safety and speed.

## Installation

You can install cutler using 🍺 [Homebrew (recommended)](https://brew.sh/):

```bash
brew install hitblast/tap/cutler
```

Besides using Homebrew as shown above, you can install the project in a couple of other ways:

1. Using `cargo`:

```bash
cargo install cutler
```

2. Using `mise`:

```bash
# NOTE: This will compile the binary manually for your system.
mise use -g cargo:cutler
```

You can also get the latest [prebuilt compressed binaries](https://github.com/hitblast/cutler/releases) if you would like to
manually install the project.

Once installed, you can install the necessary [shell completions](#shell-completions) for your shell instance if needed.

## Usage

To easily get started, simply type the following command to generate a prebuilt configuration:

```bash
cutler init
```

By default, cutler stores your configuration in `~/.config/cutler/config.toml`. But,
it can also have other values depending on your setup:

- `$XDG_CONFIG_HOME/cutler/config.toml`
- `~/.config/cutler/config.toml`
- `~/.config/cutler.toml`
- `config.toml` in the current directory (fallback)

It respects your `$XDG_CONFIG_HOME` setting, so you don't have to worry about
path issues. 

### Basic Defaults Manipulation

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

### Applying Modifications

Once you've set your preferred configurations in place, just type this one simple command:

```bash
cutler apply
```

-and cutler will automatically apply and track all of it.

To see what changes are being tracked, run:

```bash
cutler status
```

To unapply every change cutler performed with `cutler apply`, run this:

```bash
cutler unapply
```

### Running External Commands

cutler also supports running external shell commands the moment it applies the defaults.
You can define commands with simple syntax like this:

```toml
[commands.greet]
run = "echo Hello World"
```

This translates to running:

```bash
echo Hello World
```

You can also store variables in order to use them later in your custom commands:

```toml
[vars]
hostname = "darkstar"

[commands.hostname]
run = "scutil set --LocalHostName $hostname"  # or ${hostname}
sudo = true  # a more "annotated sudo"
```

By default, cutler will run all of your external commands with the `cutler apply` command if you do not pass in the
`--no-exec` flag. But, if you'd like to *only* run the commands and not apply defaults, run:

```bash
cutler exec
```

You can also run a specific external command by attaching a name parameter:

```bash
cutler exec hostname  # this runs the hostname command
```

### Config Management

Sometimes it might be handy to have a look at your current config file without having to open it. In such an event, run:

```bash
cutler config show
```

This will show all the bare-bones values that you have written. You can also delete the file if necessary:

```bash
cutler config delete
```

## Shell Completions

> [!IMPORTANT]
> If you have installed cutler using Homebrew, the shell completion will automatically be
> installed. Just restart your shell after initial installation.

#### Bash completions setup

1. Make a directory to store Bash-specific completions:

```bash
mkdir ~/.bash-completion.d/
```

2. Generate the completion script using the following command and pipe the output to a new file:

```bash
cutler completion bash > cutler.bash
mv cutler.bash ~/.bash-completion.d/
```

3. Finally, source the completion script. The best way would be to simply add it to your `.bashrc` file:

```bash
source ~/.bash_completion.d/cutler.bash > ~/.bashrc
```

#### Zsh completions setup

1. Make sure you have a directory for custom completions:

```bash
mkdir -p ~/.zfunc
```

2. Then, generate the completion script and move it over:

```bash
cutler completion zsh > _cutler
mv _cutler ~/.zfunc/
```

3. Then, add to your `~/.zshrc`:

```bash
fpath=(~/.zfunc $fpath)
autoload -U compinit && compinit
```

4. Restart your shell or run:

```bash
source ~/.zshrc
```

#### For other shells

```bash
# Fish
cutler completion fish

# Elvish
cutler completion elvish

# PowerShell
cutler completion powershell
```

## Resources

Finding the ideal set of macOS defaults can be challenging. Visit this website to have a look at
some useful ones fast: [macOS defaults website](https://macos-defaults.com/)

Sample configuration files are preincluded with this repository for you to have a look
at and get hold of the tool quickly: [see examples](./examples)

## Contributing

This is a personal project aimed at making the task of setting up a Mac more
straightforward. Contributions are always welcome! Feel free to help out by
[creating a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request) or [submitting an issue](https://github.com/hitblast/cutler/issues).

If you, as a developer, would like to dive into the nitty-gritty of contributing to cutler, view the [CONTRIBUTING.md](./CONTRIBUTING.md). I'm still writing it as the project progresses.

## License

This project is licensed under the [MIT License](https://github.com/hitblast/cutler/blob/main/LICENSE).
