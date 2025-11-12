# Installation

You can install cutler by directly running this command in the terminal:

```sh
curl -fsSL https://machlit.github.io/scripts/install.sh | /bin/bash
```

Other installation methods are:

1. Using 🍺 Homebrew:

```sh
brew install machlit/tap/cutler
```

2. Using `cargo`:

```sh
cargo install cutler
```

3. Using `mise`:

```sh
# NOTE: This will compile the binary manually for your system.
mise use -g cargo:cutler
```

Once installed, you can also enable [shell completions](./shell-integrations.md#completions) for your shell instance if needed.
Installing via Homebrew doesn't require doing this step.

## Manual Installation

Get the latest [prebuilt compressed binaries](https://github.com/machlit/cutler/releases) if you would like to manually install the project.

Note than on devices running macOS, you'll have to remove the quarantine attribute from the binary:

```sh
# inside extracted zip
xattr -d com.apple.quarantine bin/cutler
```
