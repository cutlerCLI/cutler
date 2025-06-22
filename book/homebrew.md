# Homebrew Integration

cutler can manage your Homebrew packages (formulae, casks, and taps) declaratively, right alongside your macOS settings. This page explains how to back up, restore, and synchronize your Homebrew setup using your cutler configuration file.

---

## Why use cutler for Homebrew?

- **Centralized config:** Track all your Homebrew packages in the same file as your system preferences.
- **Easy backup & restore:** Reproduce your development environment on any Mac with a single command.
- **No more Brewfile:** Avoid maintaining a separate `Brewfile` or using `brew bundle`.

---

## Homebrew Section in Config

Add a `[brew]` table to your configuration file:

```toml
[brew]
taps = ["hitblast/tap"]
casks = ["zed", "zulu@21", "android-studio"]
formulae = ["rust", "python3"]
no-deps = true # Only track manually installed packages
```

- `taps`: Additional Homebrew taps to add.
- `casks`: GUI apps and other casks to install.
- `formulae`: CLI tools and libraries to install.
- `no-deps`: If `true`, only track packages you explicitly installed (not dependencies).

---

## Backing Up Your Homebrew State

To automatically populate the `[brew]` section with your currently installed packages:

```bash
cutler brew backup
```

- This will scan your system and write all installed formulae, casks, and taps to your config file.
- To only include packages you installed directly (not dependencies):

```bash
cutler brew backup --no-deps
```

---

## Restoring Homebrew Packages

To install all formulae, casks, and taps listed in your config:

```bash
cutler brew install
```

- cutler will only install what's missing, and warn you about extra packages not tracked in your config.
- You can also run this as part of a full apply:

```bash
cutler apply --with-brew
```

---

## Keeping Your Config in Sync

If you install or remove packages outside of cutler, you can re-run:

```bash
cutler brew backup
```

to update your config file.

cutler will warn you about any "extra" packages on your system that aren't tracked in your config.

---

## Example Workflow

```bash
# Backup your current Homebrew state to config
cutler brew backup

# Edit your config to add/remove packages as desired

# Restore all packages on a new Mac
cutler brew install
```

---

## Tips

- Make sure Homebrew is installed and available in your `$PATH`.
- cutler will prompt to install Homebrew if it's missing.
- For more advanced Homebrew usage, see the [official Homebrew documentation](https://brew.sh/).

---

## More

- See [Configuration](./configuration.md) for details on the config file structure.
- See [Usage](./usage.md) for more on applying and checking your setup.
- For troubleshooting, run:

```bash
cutler status
```

to see what differs between your config and your system.

---