# Global Flags

cutler supports several global flags that can be used with any command:

- `-v`, `--verbose`: Increase output verbosity.
- `--quiet`: Suppress all output except errors and warnings. This is useful for scripting or when you only want to see problems.
- `--dry-run`: Print what would be done, but do not execute any changes.
- `-y`, `--accept-interactive`: Accept all interactive prompts automatically.
- `-n`, `--no-restart-services`: Do not restart system services after command execution.
- `--no-sync`: Do not sync with remote config (if autosync = true).

Example usage:

```sh
cutler apply --quiet
```

This will apply your configuration, but only errors and warnings will be "hushed".
