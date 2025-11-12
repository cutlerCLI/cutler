# Quickstart

To easily get started, simply type the following command to generate a sample configuration:

Most commands support a set of global flags that affect output and behavior.
See [Global Flags](./global-flags.md) for details.

```sh
cutler init
```

Once you run this command, you will get a copy of [this starter template](https://github.com/machlit/cutler/blob/master/examples/complete.toml) in the
configuration path.

## Configuration Paths

The path defaults to `$HOME/.config/cutler/config.toml` but can
be set anywhere within these locations:

- `$HOME/.config/cutler/config.toml`
- `$HOME/.config/cutler.toml`
- `$XDG_CONFIG_HOME/cutler/config.toml`
- `$XDG_CONFIG_HOME/cutler.toml`

## How to write a config?

Learn about this in the next section: [Basics & System Settings](./basics-and-system-settings.md)
