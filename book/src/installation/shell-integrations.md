# Shell Integrations

## Completions

cutler supports built-in shell completion for your ease of access for a variety of system shells, including Bash, Zsh, Powershell etc. Below you will find setup instructions to enable completions automatically for every new shell session.

> **NOTE:** If you have installed cutler using Homebrew, the shell completion will automatically be installed. Just restart your shell after initial installation.

### Bash

Run the command below:

```bash
eval "$(cutler completion bash)" > .bashrc  # or .bash_profile
```

Then restart your shell or run:

```bash
source ~/.bashrc
```

### Zsh

1. Create a directory for custom completions (if it doesn't exist):

    ```sh
    mkdir -p ~/.zfunc
    ```

2. Generate the completion script and move it:

    ```sh
    cutler completion zsh > ~/.zfunc/_cutler
    ```

3. Add the following to your `~/.zshrc`:

    ```sh
    fpath=(~/.zfunc $fpath)
    autoload -U compinit && compinit
    ```

4. Restart your shell or run:

    ```sh
    source ~/.zshrc
    ```

### Fish

Add the completion script to your fish configuration directory:

```fish
cutler completion fish > ~/.config/fish/completions/cutler.fish
```

Restart your shell or open a new fish session.

### Elvish

Add the following to your Elvish configuration file (usually `~/.elvish/rc.elv`):

```elvish
eval (cutler completion elvish)
```

Restart your shell or source your config file.

### PowerShell

Add the following to your PowerShell profile (you can find your profile path with `$PROFILE`):

```powershell
cutler completion powershell | Out-String | Invoke-Expression
```

Restart your shell or run:

```powershell
. $PROFILE
```
