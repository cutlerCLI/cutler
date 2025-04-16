#[cfg(test)]
mod tests {
    use cutler::config::load_config;
    use cutler::domains::collect_domains;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    // TODO: Config application process hasn't been included in the integration test yet because
    // currently doing that would completely blow up my Mac's configuration.
    // Need a more feasible approach.

    #[test]
    fn test_config_to_domains_workflow() {
        // Create a temporary config file
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        // Write a sample configuration
        let config_content = r#"
            [dock]
            tilesize = 50
            autohide = true

            [finder]
            ShowPathbar = true

            [NSGlobalDomain.com.apple.keyboard]
            fnState = false
        "#;

        let mut file = File::create(&config_file).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        // Load the config
        let config = load_config(&config_file).unwrap();

        // Collect domains
        let domains = collect_domains(&config).unwrap();

        // Verify the collected domains
        assert_eq!(domains.len(), 3);
        assert!(domains.contains_key("dock"));
        assert!(domains.contains_key("finder"));
        assert!(domains.contains_key("NSGlobalDomain.com.apple.keyboard"));

        // Check specific settings
        let dock = domains.get("dock").unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_integer().unwrap(), 50);
        assert_eq!(dock.get("autohide").unwrap().as_bool().unwrap(), true);

        let finder = domains.get("finder").unwrap();
        assert_eq!(finder.get("ShowPathbar").unwrap().as_bool().unwrap(), true);

        let keyboard = domains.get("NSGlobalDomain.com.apple.keyboard").unwrap();
        assert_eq!(keyboard.get("fnState").unwrap().as_bool().unwrap(), false);
    }
}
