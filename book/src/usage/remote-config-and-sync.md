# Remote Config & Sync

cutler features a simple built-in remote sync logic. For example, if you want to apply a config from a given URL without placing it manually on your machine, simply use the following command:

```sh
cutler apply --url https://example.com/config.toml
```

cutler will then download and validate the config file, and if all looks good, it will be loaded to be applied.

You can also use this functionality on your "existing" configurations, by utilizing the `[remote]` table:

```toml
[remote]
url = "https://example.com/config.toml"
autosync = true
```

Here, the `autosync` flag will ensure that the next time you run any command except the [disabled commands](#disabled-commands), it will automatically fetch the config file beforehand from the provided remote.

Or, you can simply fetch from the config URL written in `[remote]` manually using the `fetch` command:

```sh
cutler fetch
```

In order to disable remote sync behavior while running any command, use the `--no-sync` global flag:

```sh
cutler status --no-sync
```

## Disabled Commands

Some commands will not respect `autosync = true` and therefore NOT synchronize the config automatically:

- `fetch`
- `brew backup`
- `self-update`
- `check-update`
- `cookbook`
- `completion`
- `init`
- `reset`
- `config`
