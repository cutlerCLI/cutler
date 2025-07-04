# Uninstallation

Obviously, cutler is still an experimental software in heavy development, so if you would like to uninstall it, please follow these steps:

## For Script Installs

Run this command in your terminal:

```bash
$ curl -s https://raw.githubusercontent.com/hitblast/cutler/main/uninstall.sh | bash
```

## For Package Manager Installs

If you have installed cutler through a package manager, please follow the instructions that match your configuration:

1. For Homebrew:

```bash
$ brew uninstall cutler
$ brew untap hitblast/tap  # if you had only installed cutler from the tap
```

2. For `cargo`:

```bash
$ cargo uninstall cutler
```

3. For `mise`:

```bash
$ mise unuse -g cargo:cutler

# choose 'All' when pruning files if available
```
