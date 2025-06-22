# External Commands

cutler lets you define and run custom shell commands as part of your configuration. This is useful for automating setup steps, running scripts, or performing system tweaks that aren't covered by macOS preferences or Homebrew.

---

## Defining External Commands

External commands are defined in your configuration file under the `[commands]` table. Each command gets its own sub-table, and you can use variables from the `[vars]` table for substitution.

### Example

```toml
[vars]
hostname = "darkstar"

[commands.hostname]
run = "scutil --set ComputerName $hostname && scutil --set HostName $hostname && scutil --set LocalHostName $hostname"
sudo = true

[commands.wall]
run = "osascript -e 'tell application \"System Events\" to tell every desktop to set picture to \"/System/Library/Desktop Pictures/Solid Colors/Black.png\" as POSIX file'"
```

- **`run`**: The shell command to execute. Variables like `$hostname` are substituted from `[vars]`.
- **`sudo`**: (Optional) If `true`, the command is run with `sudo`.
- **`ensure-first`**: (Optional) If `true`, this command runs before all others, in order.

---

## Running External Commands

By default, all external commands are run when you execute:

```bash
cutler apply
```

You can also run only the external commands (without applying system preferences) using:

```bash
cutler exec
```

To run a specific command by name:

```bash
cutler exec hostname
```

This will run the `[commands.hostname]` entry from your config.

---

## Command Ordering

- Commands with `ensure-first = true` are run sequentially before all others.
- All other commands are run in parallel.

This allows you to control the order for commands that must be executed before others (e.g., cloning dotfiles before running setup scripts).

---

## Variable Substitution

You can use variables defined in the `[vars]` table or environment variables in your command strings:

```toml
[vars]
greeting = "Hello World"

[commands.greet]
run = "echo $greeting"
```

- `${VAR}` and `$VAR` syntax are both supported.
- If a variable is not found in `[vars]`, cutler will look for it in the environment.

---

## Sudo and Permissions

- If `sudo = true` is set, the command will be run with `sudo sh -c ...`.
- Make sure you have the necessary permissions or have configured passwordless sudo if running in automation.

---

## Dry Run

You can preview what commands would be executed without running them:

```bash
cutler exec --dry-run
```

---

## Tips

- Use external commands for tasks like setting up dotfiles, configuring system tools, or running custom scripts.
- For complex setups, break commands into logical steps and use `ensure-first` as needed.
- All output and errors from commands are logged by cutler.

---

## More

- See [Configuration](./configuration.md) for more on writing your config file.
- For advanced scripting, you can use arrays and variables for flexible command construction.

If you need help, run:

```bash
cutler --help
```

or see the [full documentation](https://hitblast.github.io/cutler/book/).