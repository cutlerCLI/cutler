<div align="center">

<img src="assets/logo.png" width="200px">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

Powerful, declarative settings management for your Mac, with speed.

[![Release Builds](https://github.com/hitblast/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/release.yml)
[![Rust Tests](https://github.com/hitblast/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/tests.yml)

</div>

> [!WARNING]
> Although cutler is solid enough for daily-driving now, expect breaking changes before the v1 release.

> [!IMPORTANT]
> The prebuilt binaries are compiled and shipped from macOS 14 on arm64.
> Intel Macs will require a manual compilation of the project.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Usage](#usage)
- [Shell Completions](#shell-completions)
- [Resources](#resources)
- [Contributing](#contributing)
- [License](#license)

## Overview

cutler simplifies macOS configuration by letting you manage system settings through a single TOML file instead of clicking through System Settings or typing complex defaults commands in the terminal.

Define your settings once, then easily apply, track, and revert changes across your system—think of it as infrastructure-as-code for your Mac.

Check out the [Usage](#usage) section for more details.

## Installation

- Install cutler using 🍺 [Homebrew](https://brew.sh/):

```bash
brew install hitblast/tap/cutler
```

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
> Currently, cutler does not verify the integrity of domains or keys under `NSGlobalDomain`. Please review these settings manually before applying any changes.

### Defaults and External Commands

cutler also supports running external shell commands the moment it applies the defaults. You can define commands with simple syntax like this:

```toml
[external]
  [[external.command]]
  cmd = "echo \"Hello World\""
```

This translates to running:

```bash
echo "Hello World"
```

For more complex scenarios, you can use a more advanced structure with separate arguments and variables:

```toml
# Define reusable variables here:
[external.variables]
common_args = ["Hello", "World"]

[external]
  [[external.command]]
  cmd = "echo"
  # If you reference a variable (for example, $common_args) and it isn't defined
  # in the [external.variables] section, cutler will fall back and try to resolve it
  # from the environment (e.g. $PATH).
  args = ["$common_args", "$PATH"]
  sudo = false
```

This roughly translates to:

```bash
echo Hello World /usr/local/bin:/usr/bin:...
```

### Applying Changes and Status Review

Once your configuration file is ready (including your defaults and external commands), apply your settings by running:

```bash
cutler apply
```

After `cutler` updates the defaults, it will also:
1. Execute any external commands defined in the `[external]` section.
2. Restart necessary system services on your Mac so that the new settings take effect.
3. Create a snapshot file named `.cutler_snapshot` in your home directory. This file records your configuration state and helps with reverting later on.

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

## Shell Completions

Here is a small tour on how to setup shell-specific completion scripts for cutler.

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
