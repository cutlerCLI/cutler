fn main() {
    #[cfg(not(target_os = "macos"))]
    {
        println!("cargo:warning=cutler is designed for macOS. Building on other platforms for development/testing purposes only.");
        println!("cargo:warning=Most functionality will be disabled on non-macOS platforms.");
    }
}
