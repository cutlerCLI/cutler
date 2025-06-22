# FAQ

Welcome to the cutler FAQ! Here you'll find answers to common questions about installation, configuration, usage, and troubleshooting.

---

## What is cutler?

cutler is a fast, declarative settings manager for macOS. It lets you manage system preferences, Homebrew packages, and custom setup commands from a single TOML configuration file.

---

## How do I install cutler?

You can install cutler with a single command:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/hitblast/cutler/main/install.sh)"
```

Or use Homebrew, Cargo, or Mise. See the [Quickstart](./quickstart.md) for details.

---

## Where does cutler store its configuration file?

cutler looks for your config file in this order:

1. `$XDG_CONFIG_HOME/cutler/config.toml`
2. `~/.config/cutler/config.toml`
3. `~/.config/cutler.toml`
4. `cutler.toml` in the current directory

You can use `cutler config show` to see which file is being used.

---

## How do I generate a starter config?

Run:

```bash
cutler init
```

This will create a sample configuration file at the appropriate location.

---

## How do I apply my configuration?

After editing your config, run:

```bash
cutler apply
```

This will apply all settings, Homebrew packages, and external commands defined in your config.

---

## How do I check what will change before applying?

Use the dry-run flag:

```bash
cutler apply --dry-run
```

Or check the current status:

```bash
cutler status
```

---

## How do I revert changes made by cutler?

Run:

```bash
cutler unapply
```

This restores all settings to their previous values (as tracked by cutler's snapshot).

---

## How do I reset everything to macOS defaults?

**Warning:** This is a destructive operation.

```bash
cutler reset --force
```

This will reset all settings defined in your config to factory defaults.

---

## How do I manage Homebrew packages with cutler?

- Use `cutler brew backup` to save your current Homebrew state to your config.
- Use `cutler brew install` to install all packages listed in your config.
- See [Homebrew Integration](./homebrew.md) for more.

---

## Can I run custom shell commands with cutler?

Yes! Define them under `[commands]` in your config. See [External Commands](./external-commands.md) for details.

---

## How do I enable shell completions?

See [Shell Integrations](./shell-integrations.md) for instructions for Bash, Zsh, Fish, Elvish, and PowerShell.

---

## How do I update cutler?

- If installed via Homebrew: `brew upgrade cutler`
- If installed via Cargo: `cargo install cutler --force`
- If installed manually: `cutler self-update`

You can check for updates with:

```bash
cutler check-update
```

---

## What if I get a permissions error or "domain does not exist"?

- Make sure you are running cutler on macOS.
- Some settings require elevated permissions; try running with `sudo` if needed.
- Double-check the domain and key names in your config.

---

## Where can I find more examples?

See the [examples directory](https://github.com/hitblast/cutler/tree/main/examples) in the GitHub repository.

---

## How can I contribute or get help?

- Open an issue or pull request on [GitHub](https://github.com/hitblast/cutler).
- See the [Contributing](./contributing.md) page for guidelines.
- For more help, run:

```bash
cutler --help
```

or visit the [full documentation](https://hitblast.github.io/cutler/book/).

---