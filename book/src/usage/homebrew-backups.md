# Homebrew Backups

If you're a person who struggles to keep tabs on all the installed formulae or apps using [Homebrew](https://brew.sh), then cutler could be a great choice for you!

## Backing up

You can back up your formula/cask names into your existng config file (or a new one) with this command:

```sh
cutler brew backup

# or, only backup the ones which are not a dependency:
#
# cutler brew backup --no-deps
```

### How it saves:

The formulae and casks are **always forced to back up in full-name convention**, meaning that if you have a formula which derives from a tap, it would be saved like this:

```
{tap}/{formula/cask}
```

And, the backup is stored in a separate `[brew]` table below the system preferences or external commands that you may have declared. So, it would look something like this:

```toml
# ~/.config/cutler/config.toml

[brew]
taps = [
    "machlit/tap"
]
casks = [
    "nikitabobko/tap/aerospace", # full-name forced wherever needed to
    "zulu@21",
    "android-studio"
]
formulae = [
    "rust",
    "machlit/tap/cutler" # same here for the formula
]

# Ensure dependencies aren't accounted for.
# This is auto-set if --no-deps is used in `brew backup`.
no_deps = true
```

## Installing

Now, when you want to install from the file, simply run:

```sh
cutler brew install
```

You can also invoke the command's functionalty from within `cutler apply`:

```sh
cutler apply --brew
```

This will install every formula/cask _alongside_ applying preferences and running external commands.

The structure of the `brew` table inside cutler's configuration is like such:

While running this command, cutler will also notify you about any extra software which is untracked by it. Then, you can run `cutler brew backup` again to sync.

## Backend Requirements (Optional)

Obviously, running Homebrew on a Mac requires the **Xcode Command-Line Tools** to be installed, let it be through Xcode itself or through
the preincluded utility in macOS. By default, cutler will try to ensure that it is there, before executing any of the subprocesses.

If you want to manually install it, you can do so by running:

```sh
xcode-select --install
```
