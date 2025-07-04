# Shell Integrations

## Completions

cutler supports built-in shell completion for your ease of access for a variety of system shells, including
`bash`, `zsh`, `powershell` etc. Below you will find instructions for each of them.

> If you have installed cutler using Homebrew, the shell completion will automatically be
> installed. Just restart your shell after initial installation.

### For `bash`

1. Make a directory to store Bash-specific completions:

```bash
mkdir ~/.bash-completion.d/
```

2. Generate the completion script using the following command and pipe the output to a new file:

```bash
cutler completion bash > cutler.bash
mv cutler.bash ~/.bash-completion.d/
```

3. Finally, source the completion script. The best way would be to simply add it to your `.bashrc` file:

```bash
source ~/.bash_completion.d/cutler.bash > ~/.bashrc
```

### For `zsh`

1. Make sure you have a directory for custom completions:

```bash
mkdir -p ~/.zfunc
```

2. Then, generate the completion script and move it over:

```bash
cutler completion zsh > _cutler
mv _cutler ~/.zfunc/
```

3. Then, add to your `~/.zshrc`:

```bash
# ~/.zshrc

fpath=(~/.zfunc $fpath)
autoload -U compinit && compinit
```

4. Restart your shell or run:

```bash
source ~/.zshrc
```

### Other Shells

```bash
# Fish
$ cutler completion fish

# Elvish
$ cutler completion elvish

# PowerShell
$ cutler completion powershell
```
