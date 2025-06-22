# Global Flags

cutler supports several global flags that can be used with any command. These flags control output verbosity, dry-run mode, interactivity, and more.

---

## Available Global Flags

You can use these flags with any cutler command:

| Flag                        | Description                                                      |
|-----------------------------|------------------------------------------------------------------|
| `-v`, `--verbose`           | Increase output verbosity. Shows more informational output.       |
| `--quiet`                   | Suppress all output except errors and warnings. Useful for scripting or when you only want to see problems. |
| `--dry-run`                 | Print what would be done, but do not execute any changes.         |
| `-y`, `--accept-interactive`| Accept all interactive prompts automatically.                     |
| `-n`, `--no-restart-services` | Do not restart system services after command execution.          |

---

## Example Usage

Apply your configuration, but only show errors and warnings:

```bash
cutler apply --quiet
```

Preview what would happen without making any changes:

```bash
cutler apply --dry-run
```

Automatically accept all prompts (useful for automation):

```bash
cutler apply --accept-interactive
```

Apply settings but do not restart system services:

```bash
cutler apply --no-restart-services
```

Combine flags as needed:

```bash
cutler apply --dry-run --quiet
```

---

## When to Use

- **Scripting/CI:** Use `--quiet` and `--accept-interactive` to suppress prompts and non-critical output.
- **Testing:** Use `--dry-run` to preview changes before applying them.
- **Advanced:** Use `--no-restart-services` if you want to manually control when system services are restarted.

---

## More

For a full list of commands and options, run:

```bash
cutler --help
```

or see the [Usage](./usage.md) page.