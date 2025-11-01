// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
    #[cfg(not(target_os = "macos"))]
    panic!("`cutler` only works on macOS and darwin-based platforms.");
}
