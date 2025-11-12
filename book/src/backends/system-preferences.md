# System Preferences Backend

<img width="150px" src="https://github.com/machlit/defaults-rs/raw/master/assets/logo.png">

## defaults-rs

In order to automate the process for setting up System Preferences, instead of relying on the `defaults` command, cutler uses
the [defaults-rs](https://github.com/machlit/defaults-rs) crate.

It communicates with the preferences daemon through the CoreFoundation API bindings in Rust. This is primarily known as the `cfprefsd` process inside macOS, which is used for setting, storing and caching preference and/or default key-value pairs.

You can view the source of the backend through the given links below. Consider contributing to make it even better for everyone!

## Project Links

- [GitHub](https://github.com/machlit/defaults-rs)
- [Documentation](https://machlit.github.io/defaults-rs)
- [crates.io](https://crates.io/crates/defaults-rs)
