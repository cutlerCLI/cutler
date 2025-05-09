#[cfg(test)]
mod tests {
    use cutler::config::{get_config_path, load_config};
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    // We need to test path resolution with environment variables
    #[test]
    fn test_get_config_path_with_env_vars() {
        // Setup a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a config directory structure
        let config_dir = temp_path.join("config");
        fs::create_dir_all(&config_dir).unwrap();

        // Create a test config file
        let config_file = config_dir.join("cutler").join("config.toml");
        fs::create_dir_all(config_file.parent().unwrap()).unwrap();
        File::create(&config_file).unwrap();

        // Set XDG_CONFIG_HOME to our temp directory
        unsafe { env::set_var("XDG_CONFIG_HOME", config_dir) };

        // Test that get_config_path returns our test file
        let config_path = get_config_path();
        assert_eq!(config_path, config_file);

        // Clean up
        unsafe { env::remove_var("XDG_CONFIG_HOME") };
    }

    #[tokio::test]
    async fn test_load_config() {
        // Setup a temporary directory and config file
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        // Write a simple TOML config
        let config_content = r#"
            [dock]
            tilesize = 46
            autohide = true
        "#;

        let mut file = File::create(&config_file).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        // Test loading the config
        let config = load_config(&config_file).await.unwrap();

        // Verify the content
        let dock = config.get("dock").unwrap().as_table().unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_integer().unwrap(), 46);
        assert!(dock.get("autohide").unwrap().as_bool().unwrap());
    }
}
