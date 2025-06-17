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
- [Global Flags](#global-flags)
- [Shell Integrations](#shell-integrations)
- [Resources](#resources)
- [Contributing](#contributing)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## Overview

cutler aims to simplify your macOS setup experience into an "almost" one-command procedure. Define your settings once, then easily apply, track, and revert changes across your system—think of it as infrastructure-as-code for your Mac.

> [!WARNING]
> This project is still under development. So, if you like it, consider starring! It's free, and it always supports me to make such projects.

Check out the [Usage](#usage) section for more details.

## Key Features

- Manage your Mac's system preferences with just one TOML file.
- Track your Homebrew formulae/casks and back them up to restore later.
- Run external commands, both as hooks, or at your will.
- Revert back modifications easily with the snapshot mechanism.
- Do all of these things at an incredibly fast speed, thanks to Rust.

## Installation

You can install cutler by directly running this command in the terminal:

```bash
$ /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/hitblast/cutler/main/install.sh)"
```

Other installation methods are:

1. Using 🍺 `homebrew`:

```bash
$ brew install hitblast/tap/cutler
```

2. Using `cargo`:

```bash
$ cargo install cutler
```

3. Using `mise`:

```bash
# NOTE: This will compile the binary manually for your system.
$ mise use -g cargo:cutler
```

Once installed, you can also enable [shell completions](#shell-completions) for your shell instance if needed.
Installing via Homebrew doesn't require doing this step.

### Manual Installation

Get the latest [prebuilt compressed binaries](https://github.com/hitblast/cutler/releases) if you would like to manually install the project.

Note than on devices running macOS, you'll have to remove the quarantine attribute from the binary:

```bash
$ xattr -d com.apple.quarantine bin/cutler  # inside extracted zip
```

## Usage

To easily get started, simply type the following command to generate a sample configuration:

Most commands support a set of global flags that affect output and behavior.
See [Global Flags](#global-flags) for details.

```bash
$ cutler init
```

By default, cutler stores your configuration in `~/.config/cutler/config.toml`.
But, it can also have other values depending on your setup:

- `$XDG_CONFIG_HOME/cutler/config.toml`
- `~/.config/cutler/config.toml`
- `~/.config/cutler.toml`
- `config.toml` in the current directory (fallback)

It respects your `$XDG_CONFIG_HOME` setting, so you don't have to worry aboutpath issues.

---

### Getting started with system preferences

cutler can do a number of things if you use it right. Here’s a basic example of a TOML configuration for cutler:

```toml
[set.dock]
tilesize = 46

[set.menuextra.clock]
FlashDateSeparators = true
```

macOS heavily relies on preference files (in `.plist` format) stored in certain ways to save the state of your Mac's apps and settings. cutler takes advantage of this mechanism to automatically put your desired system settings in place by following the config file you wrote. It's a "declarative" way to set your settings without even touching the app itself.

Ideally, the block above would look something like this if you were to manually call the `defaults` CLI tool which is used to modify these values on macOS:

```bash
$ defaults write com.apple.dock "tilesize" -int "46"
$ defaults write com.apple.menuextra.clock "FlashDateSeparators"
```

You can also configure global preferences like this:

```toml
[set.NSGlobalDomain]
InitialKeyRepeat = 15
ApplePressAndHoldEnabled = true

[set.NSGlobalDomain.com.apple.mouse]
linear = true
```

Again, if you were to use `defaults`, it would look something like this:

```bash
$ defaults write NSGlobalDomain "ApplePressAndHoldEnabled" -bool true
$ defaults write NSGlobalDomain com.apple.mouse.linear -bool true
```

Once you're ready, run this command.
In a moment, you'll see a few different system services restart as you apply the modifications you just wrote for yourself.

```bash
$ cutler apply
```

cutler also takes the changes into account and tracks them. To see your status, run:

```bash
$ cutler status
```

Unapplying everything is also as easy. Simply go ahead and run the command below and cutler will restore your preferences to the exact previous state.

```bash
$ cutler unapply
```

### Manipulating Homebrew

If you're a person who struggles to keep tabs on all the installed formulae or apps using [Homebrew](https://brew.sh), then cutler could be a great choice for you! Make sure your Homebrew installation is accessible from the `$PATH` variable, and then you can back up the necessary formula/cask names into the config file you previously wrote, using this command:

```bash
$ cutler brew backup

# or, only backup the ones which are not a dependency:
$ cutler brew backup --no-deps
```

This eliminates the usage of the notorious `brew bundle` command which creates a separate `Bundlefile` for you to track. Why do so much when all you need is just one, central file?

Now, when you want to install from the file, simply run:

```bash
$ cutler brew install
```

You can also invoke the command's functionalty from within `cutler apply`:

```bash
$ cutler apply --with-brew
```

This will install every formula/cask which is uninstalled.

The structure of the `brew` table inside cutler's configuration is like such:

```toml
[brew]
taps = ["hitblast/tap"]
casks = ["zed", "zulu@21", "android-studio"]
formulae = ["rust", "python3"]
```

While running this command, cutler will also notify you about any extra software which is untracked by it. Then, you can run `cutler brew backup` again to sync.

### Going manual with external commands

cutler also supports running external shell commands the moment it applies the defaults. This is kind of like
pre-commit git hooks where a command runs _before_ you commit anything to your project.

You can define external commands with simple syntax like this:

```toml
[commands.greet]
run = "echo Hello World"
```

This translates to running:

```bash
$ echo Hello World
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
`--no-exec` flag. But, if you'd like to _only_ run the commands and not apply defaults, run:

```bash
$ cutler exec
```

You can also run a specific external command by attaching a name parameter:

```bash
$ cutler exec hostname  # this runs the hostname command
```

### Wanna see the configuration?

Sometimes it might be handy to have a look at your current config file without having to open it. In such an event, run:

```bash
$ cutler config show
```

This will show all the bare-bones values that you have written. You can also delete the file if necessary:

```bash
$ cutler config delete
```

## Global Flags

cutler supports several global flags that can be used with any command:

- `-v`, `--verbose`: Increase output verbosity.
- `--quiet`: Suppress all output except errors and warnings. This is useful for scripting or when you only want to see problems.
- `--dry-run`: Print what would be done, but do not execute any changes.
- `-y`, `--accept-interactive`: Accept all interactive prompts automatically.
- `-n`, `--no-restart-services`: Do not restart system services after command execution.

Example usage:

```bash
cutler apply --quiet
```

This will apply your configuration, but only errors and warnings will be "hushed".

## Shell Integrations

### Completions

cutler supports built-in shell completion for your ease of access for a variety of system shells, including
`bash`, `zsh`, `powershell` etc. Below you will find instructions for each of them.

> [!IMPORTANT]
> If you have installed cutler using Homebrew, the shell completion will automatically be
> installed. Just restart your shell after initial installation.

#### Bash completions setup

1. Make a directory to store Bash-specific completions:

```bash
$ mkdir ~/.bash-completion.d/
```

2. Generate the completion script using the following command and pipe the output to a new file:

```bash
$ cutler completion bash > cutler.bash
$ mv cutler.bash ~/.bash-completion.d/
```

3. Finally, source the completion script. The best way would be to simply add it to your `.bashrc` file:

```bash
$ source ~/.bash_completion.d/cutler.bash > ~/.bashrc
```

#### Zsh completions setup

1. Make sure you have a directory for custom completions:

```bash
$ mkdir -p ~/.zfunc
```

2. Then, generate the completion script and move it over:

```bash
$ cutler completion zsh > _cutler
$ mv _cutler ~/.zfunc/
```

3. Then, add to your `~/.zshrc`:

```bash
$ fpath=(~/.zfunc $fpath)
$ autoload -U compinit && compinit
```

4. Restart your shell or run:

```bash
$ source ~/.zshrc
```

#### For other shells

```bash
# Fish
$ cutler completion fish

# Elvish
$ cutler completion elvish

# PowerShell
$ cutler completion powershell
```

## Resources

Finding the ideal set of macOS defaults can be challenging. Visit this website to have a look at
some useful ones fast:

1. [macOS defaults website](https://macos-defaults.com/)

Sample configuration files are preincluded with this repository for you to have a look
at and get hold of the tool quickly: [see examples](./examples)

## Contributing

This is a hobby project of mine which has slowly started to scale up to a full-time side project. You can always help out with new ideas or features by [creating a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request) or [submitting an issue](https://github.com/hitblast/cutler/issues)!

If you, as a developer, would like to dive into the nitty-gritty of contributing to cutler, view the [CONTRIBUTING.md](./CONTRIBUTING.md). I'm still writing it as the project progresses.

## Acknowledgements

- ^w^ Heartfelt thanks to [@furtidev](https://github.com/furtidev) for helping me learn more about the optimization process of cutler.

## License

This project is licensed under the [MIT License](https://github.com/hitblast/cutler/blob/main/LICENSE).
