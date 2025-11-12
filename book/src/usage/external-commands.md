# External Commands

Running commands to spring up your environment is essential for any workflow. Luckily, cutler is made with most scenarios in mind, given that most people usually set their dotfiles up with shell scripts which require manual execution and intervention.

You can define external commands with simple syntax like this:

```toml
# ~/.config/cutler/config.toml

[command.greet]
run = "echo Hello World"

# This runs:
# echo Hello World
```

## Variables

You can store localized variables (not available to the shell environment) inside cutler for your commands as such:

```toml
# ~/.config/cutler/config.toml

[vars]
hostname = "darkstar"

[command.hostname]
run = """
#!/usr/bin/env bash
scutil --set LocalHostName $hostname
scutil --set HostName $hostname
scutil --set ComputerName $hostname
"""
sudo = true  # a more "annotated" sudo
```

## Prioritizing Commands

Some people would like to run their commands "before" other commands. But, cutler runs all commands in parallel, which might not be what you want. In that case, you can use the `ensure_first` key to run then in your desired serial. You can apply this to multiple commands.

```toml
# ~/.config/cutler/config.toml

[command.dotfiles]
run = "git clone repo && cd repo && stow . -t ~"
ensure_first = true
```

## Ensuring Binaries

You may want to ensure that certain binaries/programs are available in `$PATH` before running an external command. You can do so with the `required` field, like this:

```toml
[command.development-tools]
run = "mise install && mise up"
required = ["mise"]  # won't run if mise is not in $PATH
```

## Running

External commands are run whenever you run `cutler apply` by default. However, if you'd like to _only_ run the commands and not apply defaults, run:

```sh
cutler exec
```

You can also run a specific external command by attaching a name parameter:

```sh
$ cutler exec hostname  # this runs the hostname command
```

## Execution Modes & Flagging

You can flag certain commands to only run when a particular flag is passed through either `apply` or `exec`. Say, if you want to flag a Hello World command:

```toml
[command.greet]
run = "echo 'Hello World'"
flag = true
```

Now that this command is flagged, it will only run if you pass one of these flags:

- `--all-cmd`/`-a`: Runs all declared commands.
- `--flagged-cmd`/`-f`: Runs flagged commands only.

```sh
$ cutler apply --all-cmd  # or -a
$ cutler apply --flagged-cmd  # or -f
```

Same goes for `cutler exec` since it also, by default, executes your commands in "regular" mode:

```sh
$ cutler exec --all  # or -r
$ cutler exec --flagged  # or -f
```
