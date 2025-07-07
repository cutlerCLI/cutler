<div align="center">

<img src="assets/logo.png" width="200px">

# <a href="https://cutlercli.github.io/">cutler</a> <sup>/kŭt′lər/</sup>

Powerful, declarative settings management for your Mac, with speed.

[![Release Builds](https://github.com/cutlerCLI/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/cutlerCLI/cutler/actions/workflows/release.yml)
[![Rust Tests](https://github.com/cutlerCLI/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/cutlerCLI/cutler/actions/workflows/tests.yml)
[![Downloads](https://img.shields.io/crates/d/cutler?style=social&logo=Rust)](https://crates.io/crates/cutler)

</div>

> [!WARNING]
> Although cutler is solid enough for daily-driving now, expect breaking changes before the v1 release.

## Overview

cutler aims to simplify your macOS setup experience into an "almost" one-command procedure. Define your settings once, then easily apply, track, and revert changes across your system—think of it as infrastructure-as-code for your Mac.

> [!IMPORTANT]
> This project is still under development. If you like it, consider starring! It's free, and it always supports me to make such projects.

## Installation

You can install cutler by running this command in the terminal:

```bash
curl -fsSL https://cutlercli.github.io/scripts/install.sh | /bin/bash
```

Other installation methods:

- **Homebrew**:
  ```bash
  brew install hitblast/tap/cutler
  ```
- **cargo**:
  ```bash
  cargo install cutler
  ```
- **mise**:
  ```bash
  mise use -g cargo:cutler
  ```

For installing manually, [see this section](https://cutlercli.github.io/cookbook/installation.html#manual-installation).

## Documentation

[**"The cutler Cookbook"**](https://cutlercli.github.io/cookbook/) should be a great starting point for anyone who wants to use this project in their setup. It is strongly encouraged to read it.

## Essential TODO

> [!NOTE]
> The list displayed below does NOT guarantee the addition of the features, but does guarantee that the developers are
> working on them, and might as well release them if maturity is reached.

- [ ] Add remote sync/orchestration features.
- [ ] Add automatic git repo cloning/management.

## Contributing

View the [Contribution Guidelines](https://cutlercli.github.io/cookbook/contributing.html) to learn more about contributing to cutler.

## License

This project is licensed under the [MIT License](https://github.com/cutlerCLI/cutler/blob/master/LICENSE).
