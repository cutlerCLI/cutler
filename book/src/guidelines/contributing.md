# Contribution Guidelines

This is the standard contribution/development guidelines for the project. You may follow these to get a hold of the project quickly.

## Table of Contents

- [Requirements](#requirements)
- [Production Workflow](#production-workflow)
- [Pull Request Guidelines](#pull-request-guidelines)
- [License](#licensing)

## Requirements

The prerequisites are as follows:

- [Rust](https://www.rust-lang.org/tools/install) (`cutler` is configured to use the 2024 edition of the language)
- A Mac (preferably with [Apple Silicon](https://support.apple.com/en-us/HT211814)) for rapid development

I would personally recommend using the latest Rust version available. As of now, I'm using Rust **v1.90** as my version.

### Cloning the Repository

Once you have ensured the prerequisites, fork the repository [from here](https://github.com/machlit/cutler/fork) and clone it using the following command:

```sh
# https
$ git clone https://github.com/<username>/cutler.git

# ssh
$ git clone git@github.com:<username>/cutler.git
```

Replace `<username>` with your GitHub username.

### Required Rust Components

Make sure your environment has these tools (NOTE: This list can change based on what currently suits the project).

- [clippy](https://github.com/rust-lang/rust-clippy)
- [rustfmt](https://github.com/rust-lang/rustfmt)

## Production Workflow

CI/CD for cutler is done using [GitHub Actions](https://docs.github.com/en/actions). You may find these workflows useful to look at:

- Release: [.github/workflows/release.yml](https://github.com/machlit/cutler/blob/master/.github/workflows/release.yml)
- Unit tests: [.github/workflows/tests.yml](https://github.com/machlit/cutler/blob/master/.github/workflows/tests.yml)

> The release workflow sets `MACOSX_DEPLOYMENT_TARGET` to `11.0`, meaning all general distributions of cutler will be compatible with **macOS Big Sur (11.0) or later versions**. You may change this according to your own needs in the workflow file as cutler is largely version-agnostic.

> The unit tests in the CI workflow are done using an **Apple Silicon M1 (3-core)** runner provided by GitHub Actions. See [this page](https://docs.github.com/en/actions/using-github-hosted-runners/using-github-hosted-runners/about-github-hosted-runners#supported-runners-and-hardware-resources) in GitHub's documentation for more information on all the runners. If the runners used in this project get outdated and don't get a bump, you may suggest one through [GitHub Issues](https://github.com/machlit/cutler/issues/new).

### Build Reproduction

You can easily create a release build for cutler using the following command:

```sh
cargo build --release --locked
```

The major part of the release automation is currently done with [GitHub Actions]() via the [following workflow](./.github/workflows/release.yml) so, you can have a look at it to view the entire pipeline.

The unit testing is done via [this workflow.](https://github.com/machlit/cutler/blob/master/.github/workflows/tests.yml)

### Code Formatting

The project uses core Rust tools to format and prettify the codebase:

```sh
# For global formatting
cargo fmt --all

# For code quality
cargo clippy --fix
```

## Pull Request Guidelines

Before submitting a pull request, please ensure the following:

- Your code is well-documented and follows the established coding standards.
- The repository is correctly forked and your working branch is up-to-date with the latest changes from the main branch.
- All tests pass locally, and you have verified that your changes do not introduce regressions.
- If your pull request fixes an issue, mention the issue number in your PR description (e.g., Fixes #123).
- For larger changes, consider discussing your approach by opening an issue first.

Pull requests and issues must have the following pattern:

```
<type>: <title>
```

Possible types include:

- feat: New feature or enhancement
- fix: Bug fix
- docs: Documentation update
- style: Code style or formatting change
- refactor: Code refactoring without changing functionality
- test: Test-related changes
- chore: Maintenance or boring tasks
