#[cfg(test)]
mod tests {
    use cutler::config::load_config;
    use cutler::domains::collect;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    // TODO: Config application process hasn't been included in the integration test yet because
    // currently doing that would completely blow up my Mac's configuration.
    // Need a more feasible approach.

    #[tokio::test]
    async fn test_config_to_domains_workflow() {
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
        let config = load_config(&config_file).await.unwrap();

        // Collect domains
        let domains = collect(&config).unwrap();

        // Verify the collected domains
        assert_eq!(domains.len(), 3);
        assert!(domains.contains_key("dock"));
        assert!(domains.contains_key("finder"));
        assert!(domains.contains_key("NSGlobalDomain.com.apple.keyboard"));

        // Check specific settings
        let dock = domains.get("dock").unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_integer().unwrap(), 50);
        assert!(dock.get("autohide").unwrap().as_bool().unwrap());

        let finder = domains.get("finder").unwrap();
        assert!(finder.get("ShowPathbar").unwrap().as_bool().unwrap());

        let keyboard = domains.get("NSGlobalDomain.com.apple.keyboard").unwrap();
        assert!(!keyboard.get("fnState").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_snapshot_integration() {
        use cutler::defaults::from_flag;
        use cutler::snapshot::state::{ExternalCommandState, SettingState, Snapshot};
        use tempfile::TempDir;

        // Create a sample snapshot
        let mut snapshot = Snapshot::new();

        // Add settings with different patterns
        snapshot.settings.push(SettingState {
            domain: "com.apple.dock".to_string(),
            key: "tilesize".to_string(),
            original_value: Some("36".to_string()),
            new_value: "46".to_string(),
        });

        snapshot.settings.push(SettingState {
            domain: "com.apple.finder".to_string(),
            key: "ShowPathbar".to_string(),
            original_value: None,
            new_value: "1".to_string(),
        });

        // Add an external command
        snapshot.external.push(ExternalCommandState {
            run: "echo \"Hello, World!\"".to_string(),
            sudo: false,
        });

        // Create a temporary directory for the test
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join(".cutler_snapshot");

        // Save the snapshot
        snapshot.save(&snapshot_path).await.unwrap();

        // Simulate what happens during unapply:
        // 1. Load the snapshot
        let loaded_snapshot = Snapshot::load(&snapshot_path).await.unwrap();

        // 2. For each setting, identify the flag and value for restoring the original value
        for setting in loaded_snapshot.settings.iter() {
            match &setting.original_value {
                Some(orig_val) => {
                    // This is what we'd do to restore the original value
                    let (flag, value) = from_flag(orig_val).unwrap();

                    // Verify the type detection works correctly
                    if orig_val == "36" {
                        assert_eq!(flag, "-int");
                        assert_eq!(value, "36");
                    }
                }
                None => {
                    // For settings that didn't exist before, we'd just delete them
                    // No assertions needed here as we're just simulating the process
                }
            }
        }

        // 3. Verify external commands are tracked correctly
        assert_eq!(loaded_snapshot.external.len(), 1);
        assert_eq!(loaded_snapshot.external[0].run, "echo \"Hello, World!\"");
    }
}
