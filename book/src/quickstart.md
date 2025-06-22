# Quickstart

Welcome to **cutler**! This page will help you get up and running in just a few minutes.

---

## Installation

Install cutler with a single command:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/hitblast/cutler/main/install.sh)"
```

Or use your favorite package manager:

- **Homebrew**:
  ```bash
  brew install hitblast/tap/cutler
  ```
- **Cargo**:
  ```bash
  cargo install cutler
  ```
- **Mise**:
  ```bash
  mise use -g cargo:cutler
  ```

For manual installation and more details, see the [Installation](./introduction.md) page.

---

## Initialize Your Configuration

To get started, generate a sample configuration file:

```bash
cutler init
```

This will create a configuration file at one of the following locations (checked in order):

- `$XDG_CONFIG_HOME/cutler/config.toml`
- `~/.config/cutler/config.toml`
- `~/.config/cutler.toml`
- `cutler.toml` in the current directory

You can customize this file to declare your preferred macOS settings, Homebrew packages, and external commands.

---

## Apply Your Settings

Once you've reviewed or edited your configuration, apply your settings with:

```bash
cutler apply
```

cutler will:

- Apply all system preferences from your config
- Track changes for easy reversion
- Optionally run Homebrew and external commands

---

## Check Status

See what differs between your config and your current system:

```bash
cutler status
```

---

## Revert Changes

To undo all changes and restore previous settings:

```bash
cutler unapply
```

---

## Next Steps

- Explore the [Configuration](./configuration.md) page for details on writing your config file.
- Learn about [Homebrew integration](./homebrew.md) and [external commands](./external-commands.md).
- For advanced usage, see [Advanced Topics](./advanced.md).

If you need help, run:

```bash
cutler --help
```

or check the [full documentation](https://hitblast.github.io/cutler/book/).

---