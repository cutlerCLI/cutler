# Backend for Preferences

In cutler's earlier versions, it used to wrap around the `defaults` CLI using asynchronous command invocations. However, this had a counterpart of being slow and extensive in terms of I/O and shell command execution.

<div align="center">

<img width="200px" src="https://github.com/hitblast/defaults-rs/raw/master/assets/logo.png">

# defaults-rs

</div>

cutler's latest versions include <a href="https://github.com/hitblast/defaults-rs">defaults-rs</a>, a frontend for macOS preferences which has been directly written for interop with cutler. Instead of command invocation, it parses the preferences and can also modify them in batches, all the while minimizing the amount of I/O per call.

It has a neat little Rust API which is also asynchronous.

This gives some visible benefits:

1. Instead of lag between commands, now you get instantaneous modifications to your settings (use `cutler apply -v` to feel the speed).

2. Granular control over what gets modified and what doesn't.

3. The community gets to modify and upgrade the backend.

## Project Links

- [crates.io](https://crates.io/crates/defaults-rs)
- [GitHub](https://github.com/hitblast/defaults-rs)
