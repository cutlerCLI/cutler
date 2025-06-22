# Advanced Topics

This page covers advanced usage patterns, tips, and features for power users of **cutler**. If you're looking to automate, customize, or deeply integrate cutler into your workflow, this is the place to start.

---

## Customizing Configuration Paths

By default, cutler looks for your configuration file in the following order:

1. `$XDG_CONFIG_HOME/cutler/config.toml`
2. `~/.config/cutler/config.toml`
3. `~/.config/cutler.toml`
4. `cutler.toml` in the current directory

If you want to use a custom path, you can symlink or copy your preferred config file to one of these locations.

---

## Using Snapshots

cutler automatically creates a snapshot of your system's previous state before applying changes. This allows you to:

- **Revert**: Use `cutler unapply` to restore all settings to their previous values.
- **Audit**: The snapshot file (`~/.cutler_snapshot`) is a JSON file containing all tracked changes and external commands run.

**Tip:** You can back up or version-control your snapshot file for extra safety.

---

## Dry-Run Mode

Preview all changes without making any modifications:

```bash
cutler apply --dry-run
```

This is useful for:

- Testing your configuration before applying it
- Scripting and CI pipelines
- Auditing what would change

---

## Automating with Scripts and CI

You can use cutler in automation by combining global flags:

- `--quiet` to suppress non-critical output
- `--accept-interactive` to auto-accept prompts
- `--dry-run` for previewing changes

Example for CI:

```bash
cutler apply --quiet --accept-interactive
```

---

## Selective Application and Execution

- **Skip external commands:**  
  ```bash
  cutler apply --no-exec
  ```
- **Only run Homebrew installs:**  
  ```bash
  cutler brew install
  ```
- **Only run external commands:**  
  ```bash
  cutler exec
  ```

---

## Forcing a Hard Reset

If you want to reset all settings defined in your config to macOS factory defaults (dangerous!):

```bash
cutler reset --force
```

**Warning:** This cannot be undone. Use only if `cutler unapply` is not possible.

---

## Manual Installation and Updates

- **Manual install:** Download the latest release from [GitHub Releases](https://github.com/hitblast/cutler/releases).
- **Self-update:**  
  ```bash
  cutler self-update
  ```
  (Only for manual installs. If installed via Homebrew or Cargo, use your package manager.)

---

## Troubleshooting

- **Check which config is being used:**  
  ```bash
  cutler config show
  ```
- **Check status and diffs:**  
  ```bash
  cutler status
  ```
- **Verbose output:**  
  ```bash
  cutler apply --verbose
  ```

---

## Tips & Best Practices

- **Version-control your config file** for reproducible setups.
- **Use comments** (`# ...`) in your TOML for documentation.
- **Keep your Homebrew section in sync** by running `cutler brew backup` after installing/removing packages.
- **Use variables** in `[vars]` for DRY (Don't Repeat Yourself) configs.
- **Group related commands** and use `ensure-first` for ordering.
- **Read the [examples directory](https://github.com/hitblast/cutler/tree/main/examples)** for real-world configs.

---

## More

- For a full list of commands and options, run:
  ```bash
  cutler --help
  ```
- For shell completions, see [Shell Integrations](./shell-integrations.md).
- For contributing and development, see [Contributing](./contributing.md).

---