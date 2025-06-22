<div align="center">

<img src="assets/logo.png" width="200px">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

Powerful, declarative settings management for your Mac, with speed.

[![Release Builds](https://github.com/hitblast/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/release.yml)
[![Rust Tests](https://github.com/hitblast/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/tests.yml)

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
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/hitblast/cutler/main/install.sh)"
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

Manual installation and more details are available in the [documentation](https://hitblast.github.io/cutler/book/).

## Documentation

[The cutler Cookbook](https://hitblast.github.io/cutler/book/) should be a great starting point for anyone who wants to use this project in their setup. I strongly encourage to read it.

## Contributing

If you, as a developer, would like to dive into the nitty-gritty of contributing to cutler, view the [contribution guidelines](https://hitblast.github.io/cutler/book/contributing.html).


## License

This project is licensed under the [MIT License](https://github.com/hitblast/cutler/blob/main/LICENSE).
