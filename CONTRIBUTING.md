# Contribution Guidelines

This is the standard contribution/development guidelines for the project. You may follow these to get a hold of the project quickly.

## Table of Contents

- [Getting Started](#getting-started)
  - [Cloning the repository](#cloning-the-repository)
  - [Preparing the environment](#preparing-the-environment)
- [Production Release Workflow](#production-release-workflow)
  - [Testing](#testing)
  - [Building](#building)
- [Project Hierarchy](#project-hierarchy)
- [Licensing](#licensing)

## Getting Started

The commonplace of contributing is to first clone the repository and install the dependencies.

The prerequisites are as follows:

- [Rust](https://www.rust-lang.org/tools/install) (`cutler` is configured to use the 2024 edition of the language)
- or, [mise (recommended)](https://mise.jdx.dev) for automatic tools management
- A Mac (preferably with [Apple Silicon](https://support.apple.com/en-us/HT211814)) for rapid development

### Cloning the repository

Once you have ensured the prerequisites, fork the repository [from here](https://github.com/hitblast/cutler/fork) and clone it using the following command:

```bash
# https
git clone https://github.com/<your_username>/cutler.git

# ssh
git clone git@github.com:<your_username>/cutler.git
```

Replace `<your_username>` with your GitHub username.

### Preparing the environment

Working on this project will require a few Rust components beforehand:

- [clippy](https://github.com/rust-lang/rust-clippy)
- [rustfmt](https://github.com/rust-lang/rustfmt)

If you're using mise, prepare the environment by running:

```bash
mise install
```

## Production Release Workflow

This chain of commands can be used to fully test and build the final product.

#### Testing

```bash
cargo fmt --all -- --check && cargo test --verbose && cargo clippy && cargo build
cargo build

# or, you can use the predefined testsuite:
mise run testsuite
```

#### Building

```bash
cargo build --release --verbose --locked

# or, you can use the predefined build script:
mise run build
```

## Project Hierarchy

When running `tree src` over the source tree, we can see that there are a few important files:

```
src
├── commands.rs
├── config.rs
├── defaults.rs
├── domains.rs
├── external.rs
├── lib.rs
├── logging.rs
└── main.rs
```

Among this, a few essential files can be highlighted:

- `main.rs`: This houses the entry point of the application and ***should NOT contain any unnecessary business logic.***
- `commands.rs`: The backend functions for each shell command cutler has is housed here, and collects with the rest of the application.
- `config.rs`: The project relies heavily on configuration file management. So, logic related to creating the configuration file and validating it is kept here.
- `domains.rs`: This file is used to wrap around the `defaults` CLI tool of macOS and provides a high-level interface to performing various I/O operations.
- `external.rs`: cutler's external command-running functionality for multiple use-cases is implemented here.
- `logging.rs`: Pretty-printing!!!

The `lib.rs` file in the source tree does not contain any logic, rather it is used to connect the various modules together.

## Code Formatting

`cutler` uses basic Rust formatting for code reliability and maintainability. This ensures that the codebase remains clean, readable, and consistent across different contributors.

Simply run the following command to format the code:

```
cargo fmt
```

For catching syntactic errors, you can use the `cargo clippy` command:

```
cargo clippy -- -D warnings
```

## Licensing

This project is licensed under the MIT License - see the [LICENSE](https://github.com/hitblast/cutler/blob/main/LICENSE) file for details.
