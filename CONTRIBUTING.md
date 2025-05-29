# Contribution Guidelines

This is the standard contribution/development guidelines for the project. You may follow these to get a hold of the project quickly.

## Table of Contents

- [Getting Started](#getting-started)
- [Production Release Workflow](#production-release-workflow)
- [License](#licensing)

## Getting Started

The commonplace of contributing is to first clone the repository and install the dependencies.

The prerequisites are as follows:

- [Rust](https://www.rust-lang.org/tools/install) (`cutler` is configured to use the 2024 edition of the language)
- A Mac (preferably with [Apple Silicon](https://support.apple.com/en-us/HT211814)) for rapid development

### Cloning the repository

Once you have ensured the prerequisites, fork the repository [from here](https://github.com/hitblast/cutler/fork) and clone it using the following command:

```bash
# https
$ git clone https://github.com/<your_username>/cutler.git

# ssh
$ git clone git@github.com:<your_username>/cutler.git
```

Replace `<your_username>` with your GitHub username.

### Preparing the environment

Working on this project will require a few Rust components beforehand:

- [clippy](https://github.com/rust-lang/rust-clippy)
- [rustfmt](https://github.com/rust-lang/rustfmt)

## Production Release Workflow

This chain of commands can be used to fully test and build the final product.

### Testing

```bash
$ cargo fmt --all -- --check && cargo test --verbose && cargo clippy && cargo build
```

> [!NOTE]
> The unit tests in the CI workflow are done using an **Apple Silicon M1 (3-core)** runner provided by GitHub Actions. See [this page](https://docs.github.com/en/actions/using-github-hosted-runners/using-github-hosted-runners/about-github-hosted-runners#supported-runners-and-hardware-resources) in GitHub's documentation for more information on all the runners. If the runners used in this project get outdated and don't get a bump, you may suggest one through [GitHub Issues](https://github.com/hitblast/cutler/issues/new).

### Build Reproduction

You can easily create a release build for cutler using the following command:

```bash
$ cargo build --release --verbose --locked
```

The major part of the release automation is currently done with [GitHub Actions]() via the [following workflow](./.github/workflows/release.yml) so, you can have a look at it to view the entire pipeline.

The unit testing is done via [this workflow.](./.github/workflows/tests.yml)

### Code Formatting

`cutler` uses basic Rust formatting for code reliability and maintainability. This ensures that the codebase remains clean, readable, and consistent across different contributors.

Simply run the following command to format the code:

```bash
$ cargo fmt --all
```

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/hitblast/cutler/blob/main/LICENSE) file for details.
