# Configuration Management

cutler's configuration can be tiny or versatile depending on your needs. But, there are some nifty features built into the software for your convenience.

## Config-Locking

When you run cutler init, the configuration file will usually contain this key-value pair at the very top:

```toml
lock = true
```

Unless you remove it, this will happen:

```bash
$ cutler apply
[ERROR] The config file is locked. Run `cutler config unlock` to unlock.
$
```

You can use this feature to mark configurations as potentially unsafe to apply. cutler uses it to generate new configuration files for you so that you don't accidentally apply the sample.

There are two commands to manage the lock status:

```bash
$ cutler config lock
# or
$ cutler config unlock
```

## View or Delete

To view your cutler configuration without the use of `cat` or any other fancy tools, use:

```bash
$ cutler config show
```

You can also delete it using:

```bash
$ cutler config delete
```
