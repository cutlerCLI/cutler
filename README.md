<!-- SPDX-License-Identifier: Apache-2.0 -->

<div align="center">

<img src="assets/logo.png" width="200px">

# <a href="https://cutlercli.github.io/">cutler</a> <sup>/kŭt′lər/</sup>

Powerful, declarative settings management for your Mac, with speed.

[![Crates.io Downloads](https://img.shields.io/crates/d/cutler?style=social&logo=Rust)](https://crates.io/crates/cutler)

</div>

## Overview

cutler aims to simplify your Mac's setup process into a one-command procedure. It does so with the following goals in mind:

1. Automating the setup for system preferences (no more manual tinkering with the Settings app).
2. Automating the installation of apps and tools (through `brew` and other tools).
3. Automating the execution of custom commands (this is on you, but cutler makes it easier).

cutler splits a single TOML file into readable configuration which you can design as your desire, allowing you to
later apply it with just one command.

## Installation

You can install cutler by running this command in the terminal:

```bash
curl -fsSL https://cutlercli.github.io/scripts/install.sh | /bin/bash
```

See ["Installation"](https://cutlercli.github.io/cookbook/installation.html) for other methods.

> [!WARNING]
> **DEPRECATION:** The x86_64 builds will soon be removed in favor of Apple Silicon, as Apple themselves have officially discontinued this timed architecture.

## Documentation

**The Cookbook** should be a great starting point for anyone who wants to use cutler. If you have just installed it, simply run:

```bash
cutler cookbook
```

This will redirect you to the online copy in browser. Or, visit: https://cutlercli.github.io/cookbook/

## Contributing

View the [Contribution Guidelines](https://cutlercli.github.io/cookbook/contributing.html) to learn more about contributing to cutler.

## License

This project is licensed under the [Apache 2.0 License](https://github.com/cutlerCLI/cutler/blob/master/LICENSE.md).
