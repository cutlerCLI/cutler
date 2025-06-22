# Basic Usage

To easily get started, simply type the following command to generate a sample configuration:

Most commands support a set of global flags that affect output and behavior.
See [Global Flags](./global-flags.md) for details.

```bash
$ cutler init
```

By default, cutler stores your configuration in `~/.config/cutler/config.toml`.
But, it can also have other values depending on your setup:

- `$XDG_CONFIG_HOME/cutler/config.toml`
- `~/.config/cutler/config.toml`
- `~/.config/cutler.toml`
- `cutler.toml` in the current directory

It respects your `$XDG_CONFIG_HOME`, so you don't have to worry about path issues.