// SPDX-License-Identifier: MIT

fn main() {
    #[cfg(not(target_os = "macos"))]
    panic!("`cutler` only works on macOS and darwin-based platforms.");
}
