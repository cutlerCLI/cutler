# Contributing

Thank you for your interest in contributing to **cutler**! This project welcomes contributions of all kinds, including code, documentation, bug reports, feature requests, and ideas.

---

## Getting Started

To contribute to cutler, you'll need:

- [Rust](https://www.rust-lang.org/tools/install) (cutler uses the 2024 edition)
- A Mac (preferably with [Apple Silicon](https://support.apple.com/en-us/HT211814)) for rapid development

### Cloning the Repository

1. **Fork** the repository on GitHub: [https://github.com/hitblast/cutler/fork](https://github.com/hitblast/cutler/fork)
2. **Clone** your fork:

   ```bash
   # HTTPS
   git clone https://github.com/<your_username>/cutler.git

   # SSH
   git clone git@github.com:<your_username>/cutler.git
   ```

   Replace `<your_username>` with your GitHub username.

### Preparing the Environment

Install the following Rust components:

- [clippy](https://github.com/rust-lang/rust-clippy)
- [rustfmt](https://github.com/rust-lang/rustfmt)

You can install them with:

```bash
rustup component add clippy rustfmt
```

---

## Development Workflow

### Testing

Before pushing changes, run:

```bash
cargo fmt --all -- --check && cargo test --verbose && cargo clippy && cargo build
```

You can automate this with [hookman](https://github.com/hitblast/hookman):

```bash
hookman build
```

> **Note:** CI tests run on Apple Silicon M1 (3-core) runners via GitHub Actions.

### Building

To create a release build:

```bash
cargo build --release --verbose --locked
```

Release automation is handled via [GitHub Actions](https://github.com/hitblast/cutler/blob/main/.github/workflows/release.yml).

### Code Formatting

Run:

```bash
cargo fmt --all
```

---

## Pull Request Guidelines

Before submitting a pull request:

- Ensure your code is well-documented and follows the project's coding standards.
- Keep your branch up-to-date with the latest changes from `main`.
- All tests must pass locally.
- If your PR fixes an issue, mention it in the description (e.g., Fixes #123).
- For larger changes, consider opening an issue to discuss your approach first.

**PR/Issue title format:**

```
(<type>) <title>
```

Where `<type>` is one of:

- feat: New feature or enhancement
- fix: Bug fix
- docs: Documentation update
- style: Code style or formatting change
- refactor: Code refactoring without changing functionality
- test: Test-related changes
- chore: Maintenance or administrative tasks

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for helping make cutler better! If you have any questions, open an issue or reach out on GitHub.