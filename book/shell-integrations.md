# Shell Integrations

cutler supports built-in shell completion for a variety of system shells, making it easier and faster to use in your terminal. This page explains how to enable and use shell completions for different shells.

---

## Supported Shells

- **Bash**
- **Zsh**
- **Fish**
- **Elvish**
- **PowerShell**

> **Note:** If you installed cutler using Homebrew, shell completions are installed automatically. Just restart your shell after installation.

---

## Bash Completions

1. **Create a directory for completions (if needed):**

    ```bash
    mkdir -p ~/.bash-completion.d/
    ```

2. **Generate the completion script:**

    ```bash
    cutler completion bash > cutler.bash
    mv cutler.bash ~/.bash-completion.d/
    ```

3. **Source the completion script in your `.bashrc`:**

    ```bash
    echo 'source ~/.bash-completion.d/cutler.bash' >> ~/.bashrc
    source ~/.bashrc
    ```

---

## Zsh Completions

1. **Create a directory for custom completions (if needed):**

    ```bash
    mkdir -p ~/.zfunc
    ```

2. **Generate the completion script:**

    ```bash
    cutler completion zsh > _cutler
    mv _cutler ~/.zfunc/
    ```

3. **Add to your `~/.zshrc`:**

    ```bash
    fpath=(~/.zfunc $fpath)
    autoload -U compinit && compinit
    source ~/.zshrc
    ```

---

## Fish Completions

Generate and use the completion script:

```bash
cutler completion fish > ~/.config/fish/completions/cutler.fish
```

---

## Elvish Completions

Generate and use the completion script:

```bash
cutler completion elvish > ~/.elvish/lib/cutler.elv
```

---

## PowerShell Completions

Generate and use the completion script:

```bash
cutler completion powershell > cutler.ps1
# Then, source or import the script in your PowerShell profile
```

---

## Usage

Once completions are installed and sourced, you can use <kbd>Tab</kbd> to auto-complete cutler commands, flags, and arguments in your shell.

---

## Troubleshooting

- If completions do not work, ensure the completion script is sourced in your shell profile and that your shell supports programmable completion.
- Restart your terminal session after making changes to your shell configuration files.

---

For more help, see the [Quickstart](./quickstart.md) or run:

```bash
cutler --help
```
