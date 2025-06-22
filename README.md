<div align="center">

<img src="assets/logo.png" width="200px">

# <img src="https://raw.githubusercontent.com/github/explore/80688e429a7d4ef2fca1e82350fe8e3517d3494d/topics/rust/rust.png" width="40px"> cutler

Powerful, declarative settings management for your Mac, with speed.

[![Release Builds](https://github.com/hitblast/cutler/actions/workflows/release.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/release.yml)
[![Rust Tests](https://github.com/hitblast/cutler/actions/workflows/tests.yml/badge.svg)](https://github.com/hitblast/cutler/actions/workflows/tests.yml)

</div>

> [!WARNING]
> Although cutler is solid enough for daily-driving now, expect breaking changes before the v1 release.

---

## Overview

cutler aims to simplify your macOS setup experience into an "almost" one-command procedure. Define your settings once, then easily apply, track, and revert changes across your system—think of it as infrastructure-as-code for your Mac.

> This project is still under development. If you like it, consider starring! It's free, and it always supports me to make such projects.

---

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
- **Cargo**:
  ```bash
  cargo install cutler
  ```
- **Mise**:
  ```bash
  mise use -g cargo:cutler
  ```

Manual installation and more details are available in the [documentation](https://hitblast.github.io/cutler/book/).

---

## Contributing

This is a hobby project of mine which has slowly started to scale up to a full-time side project. You can always help out with new ideas or features by [creating a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request) or [submitting an issue](https://github.com/hitblast/cutler/issues)!

If you, as a developer, would like to dive into the nitty-gritty of contributing to cutler, view the [CONTRIBUTING.md](./CONTRIBUTING.md).

---

## License

This project is licensed under the [MIT License](https://github.com/hitblast/cutler/blob/main/LICENSE).

---

## Documentation

Full documentation, usage, quickstart, and advanced guides are available at:

- [cutler Book (mdBook)](https://hitblast.github.io/cutler/book/)

