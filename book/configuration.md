# Configuration

cutler uses a single TOML configuration file to declaratively manage your macOS settings, Homebrew packages, and external commands. This page explains the structure and options available in your config file.

---

## Where is the config file?

cutler looks for your configuration file in the following locations (in order):

1. `$XDG_CONFIG_HOME/cutler/config.toml`
2. `~/.config/cutler/config.toml`
3. `~/.config/cutler.toml`
4. `cutler.toml` in the current directory

You can generate a starter config with:

```bash
cutler init
```

---

## Basic Structure

A minimal configuration might look like this:

```toml
[set.finder]
AppleShowAllFiles = true
CreateDesktop = false
ShowPathbar = true

[set.dock]
tilesize = 46
autohide = true
orientation = "left"
```

- The `[set.DOMAIN]` tables correspond to macOS preference domains.
- Keys and values inside each table are the settings to apply.

---

## Advanced Example

Here's a more comprehensive example:

```toml
[set.menuextra.clock]
FlashDateSeparators = true
DateFormat = "\"h:mm:ss\""
Show24Hour = false
ShowAMPM = false
ShowDate = 2
ShowDayOfWeek = false
ShowSeconds = true

[set.finder]
AppleShowAllFiles = true
CreateDesktop = false
ShowPathbar = true
ShowExternalHardDrivesOnDesktop = false

[set.dock]
tilesize = 50
autohide = true
static-only = true
show-recents = false
magnification = false
orientation = "right"
mineffect = "suck"
autohide-delay = 0
autohide-time-modifier = 0.6
expose-group-apps = true

[set.NSGlobalDomain]
InitialKeyRepeat = 15
ApplePressAndHoldEnabled = true
"com.apple.mouse.linear" = true

[set.NSGlobalDomain.com.apple.keyboard]
fnState = false
```

---

## Homebrew Integration

You can track Homebrew formulae, casks, and taps in your config:

```toml
[brew]
taps = ["hitblast/tap"]
casks = ["zed", "zulu@21", "android-studio"]
formulae = ["rust", "python3"]
no-deps = true # Only track manually installed packages
```

- Use `cutler brew backup` to automatically populate this section from your system.
- Use `cutler brew install` to install all listed packages.

---

## External Commands

Define custom shell commands to run as part of your setup:

```toml
[vars]
hostname = "darkstar"

[commands.hostname]
run = "scutil --set ComputerName $hostname && scutil --set HostName $hostname && scutil --set LocalHostName $hostname"
sudo = true

[commands.wall]
run = "osascript -e 'tell application \"System Events\" to tell every desktop to set picture to \"/System/Library/Desktop Pictures/Solid Colors/Black.png\" as POSIX file'"
```

- Use `[vars]` to define reusable variables for substitution.
- Each `[commands.NAME]` table must have a `run` key (the shell command).
- `sudo = true` runs the command with `sudo`.
- `ensure-first = true` runs the command before others, in order.

---

## Tips

- You can comment out lines with `#`.
- Arrays and nested tables are supported.
- For more examples, see the [examples directory](https://github.com/hitblast/cutler/tree/main/examples).

---

## Next Steps

- See [Usage](./usage.md) for how to apply, check, and revert your configuration.
- Learn about [Homebrew integration](./homebrew.md) and [external commands](./external-commands.md).
- For advanced features, see [Advanced Topics](./advanced.md).