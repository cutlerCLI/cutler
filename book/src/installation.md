# Installation

You can install cutler by directly running this command in the terminal:

```bash
$ /bin/sh -c "$(curl -fsSL https://raw.githubusercontent.com/hitblast/cutler/main/install.sh)"
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

Once installed, you can also enable [shell completions](./shell-integrations.md#completions) for your shell instance if needed.
Installing via Homebrew doesn't require doing this step.

## Manual Installation

Get the latest [prebuilt compressed binaries](https://github.com/hitblast/cutler/releases) if you would like to manually install the project.

Note than on devices running macOS, you'll have to remove the quarantine attribute from the binary:

```bash
$ xattr -d com.apple.quarantine bin/cutler  # inside extracted zip
```
