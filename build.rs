// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() {
    #[cfg(not(target_os = "macos"))]
    panic!("cutler only works on macOS.");
}
