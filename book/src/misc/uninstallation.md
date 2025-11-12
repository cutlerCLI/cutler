# Uninstallation

Obviously, cutler is still an experimental software in heavy development, so if you would like to uninstall it, please follow these steps:

## For Script Installs

Run this command in your terminal:

```sh
curl -fsSL https://machlit.github.io/scripts/uninstall.sh | /bin/bash
```

## For Package Manager Installs

If you have installed cutler through a package manager, please follow the instructions that match your configuration:

1. For Homebrew:

```sh
brew uninstall cutler
brew untap machlit/tap  # if you had only installed cutler from the tap
```

2. For `cargo`:

```sh
cargo uninstall cutler
```

3. For `mise`:

```sh
mise unuse -g cargo:cutler

# when prompted,
# choose 'All' when pruning files if available
```
