# Usage

This page covers the core usage patterns for **cutler**. Once you have installed cutler and initialized your configuration, you can manage your Mac's settings, Homebrew packages, and external commands with a few simple commands.

---

## Basic Workflow

1. **Initialize your configuration:**

   ```bash
   cutler init
   ```

   This creates a sample configuration file. Edit this file to declare your desired settings.

2. **Apply your configuration:**

   ```bash
   cutler apply
   ```

   This command applies all settings, Homebrew packages, and external commands defined in your config file.

3. **Check your system status:**

   ```bash
   cutler status
   ```

   Compares your current system state with your configuration and shows any differences.

4. **Revert changes:**

   ```bash
   cutler unapply
   ```

   Restores all settings to their previous values (as tracked by cutler's snapshot).

---

## Command Reference

### `cutler apply`

- Applies all system preferences, Homebrew packages, and external commands from your config.
- Tracks changes for easy reversion.
- Options:
  - `--no-exec`: Skip running external commands.
  - `--with-brew`: Also run `cutler brew install` after applying settings.
  - `--disable-checks`: (Advanced) Skip domain existence checks.

### `cutler status`

- Shows which settings, Homebrew packages, or taps are missing or extra compared to your config.
- Useful for auditing your system or before applying changes.

### `cutler unapply`

- Reverts all changes made by the last `cutler apply`.
- Restores previous values using the snapshot mechanism.

### `cutler reset --force`

- **Dangerous:** Resets all settings defined in your config file to macOS factory defaults.
- Use only as a last resort if `cutler unapply` is not possible.

### `cutler exec [NAME]`

- Runs all external commands defined in your config, or a specific command if `NAME` is provided.
- Example:

  ```bash
  cutler exec greet
  ```

  Runs the `[commands.greet]` entry from your config.

### `cutler config show`

- Displays the contents of your current configuration file.

### `cutler config delete`

- Deletes your configuration file and unapplies any settings if a snapshot exists.

---

## Example Workflow

```bash
# Initialize config
cutler init

# Edit ~/.config/cutler/config.toml (or your config path)

# Apply all settings
cutler apply

# Check what differs from your config
cutler status

# Revert all changes
cutler unapply

# Hard reset to macOS defaults (dangerous)
cutler reset --force
```

---

## Where is the config file?

cutler looks for your configuration file in the following order:

1. `$XDG_CONFIG_HOME/cutler/config.toml`
2. `~/.config/cutler/config.toml`
3. `~/.config/cutler.toml`
4. `cutler.toml` in the current directory

You can use `cutler config show` to see which file is being used.

---

## More

- For details on writing your config, see [Configuration](./configuration.md).
- For Homebrew integration, see [Homebrew Integration](./homebrew.md).
- For external commands, see [External Commands](./external-commands.md).
- For global flags and advanced usage, see [Global Flags](./global-flags.md) and [Advanced Topics](./advanced.md).

If you need help at any time, run:

```bash
cutler --help
```

or visit the [full documentation](https://hitblast.github.io/cutler/book/).