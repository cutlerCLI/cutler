# Configuration Management

cutler's configuration can be tiny or versatile depending on your needs. But, there are some nifty features built into the software for your convenience.

## Config-locking

When you run cutler init, the configuration file will usually contain this key-value pair at the very top:

```toml
lock = true
```

Unless you remove it, this will happen:

```bash
$ cutler apply
[ERROR] The config is locked. Remove the `lock = true` line to apply this config.
$
```

You can use it to mark configurations as potentially unsafe to apply. cutler uses it to also generate new configuration files for you (without the risk of you accidentally applying it).

## Commands

```bash
$ cutler config show
```

This will show all the bare-bones values that you have written. You can also delete the file if necessary:

```bash
$ cutler config delete
```
