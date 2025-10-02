<!-- SPDX-License-Identifier: Apache-2.0 -->

<div align="center">

<img src="assets/logo.png" width="200px">

# <a href="https://cutlercli.github.io/">cutler</a> <sup>/kŭt′lər/</sup>

Powerful, declarative settings management for your Mac, with speed.

[![Crates.io Downloads](https://img.shields.io/crates/d/cutler?style=social&logo=Rust)](https://crates.io/crates/cutler)

</div>

## Overview

cutler aims to simplify your Mac's setup process into a one-command procedure. It does so by automating these and more:

1. **System preferences.** No more tinkering with the Settings app.
2. Installation of **apps and tools** (through `brew` and other tools).
3. The execution of **custom commands** (this is on you, but cutler makes it super convenient!).

cutler splits a single TOML file into readable configuration which you can design as your desire, allowing you to
later apply it with just `cutler apply`!

## Installation

Copy and run the command below to install cutler:

```bash
curl -fsSL https://cutlercli.github.io/scripts/install.sh | /bin/bash
```

Or, see ["Installation"](https://cutlercli.github.io/cookbook/installation.html) for other installation methods.

> [!WARNING]
> **DEPRECATION:** The x86_64 builds will soon be removed in favor of Apple Silicon, as Apple themselves have officially discontinued this timed architecture.

## Documentation

["The cutler Cookbook"](https://cutlercli.github.io/cookbook) can be a great starting point for in-depth syntax review and examples if you want to use this tool.

Run the command below to open it in your browser:

```bash
cutler cookbook
```

## Contributing

View the [Contribution Guidelines](https://cutlercli.github.io/cookbook/contributing.html) to learn more about contributing to cutler. It also contains resources such as code snippets to make your contribution workflow easier.

## License

This project is licensed under the [Apache 2.0 License](https://github.com/cutlerCLI/cutler/blob/master/LICENSE.md).
