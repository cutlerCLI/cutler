# Configuration Features

There are some nifty features built into the software for your convenience. These configuration features have been documented below so that you can have a quick look:

## Config-Locking

When you run cutler init, the configuration file will usually contain this key-value pair at the very top:

```toml
# ~/.config/cutler/config.toml

lock = true
...
```

Unless you remove it, this will happen:

```sh
$ cutler apply
ERR   The config file is locked. Run `cutler unlock` to unlock.
$
```

You can use this feature to mark configurations as potentially unsafe to apply. cutler uses it to generate new configuration files for you so that you don't accidentally apply the sample.

From your terminal, it's also as easy to lock:

```sh
cutler lock
```

and, to unlock:

```sh
cutler unlock
```
