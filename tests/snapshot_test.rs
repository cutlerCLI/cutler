#[cfg(test)]
mod tests {
    use cutler::snapshot::{ExternalCommandState, SettingState, Snapshot, get_snapshot_path};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::{env, fs};
    use tempfile::TempDir;

    #[test]
    fn test_get_snapshot_path() {
        // Setup a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Set HOME to our temp directory
        unsafe { env::set_var("HOME", temp_path.to_str().unwrap()) };

        // Test that get_snapshot_path returns .cutler_snapshot in the home directory
        let snapshot_path = get_snapshot_path();
        assert_eq!(
            snapshot_path,
            PathBuf::from(temp_path).join(".cutler_snapshot")
        );

        // Clean up
        unsafe { env::remove_var("HOME") };
    }

    #[test]
    fn test_snapshot_basic() {
        // Test creation
        let snapshot = Snapshot::new();
        assert_eq!(snapshot.settings.len(), 0);
        assert_eq!(snapshot.external_commands.len(), 0);
        assert_eq!(snapshot.version, env!("CARGO_PKG_VERSION"));

        // Test setting state
        let setting = SettingState {
            domain: "com.apple.dock".to_string(),
            key: "tilesize".to_string(),
            original_value: Some("36".to_string()),
            new_value: "46".to_string(),
        };
        assert_eq!(setting.domain, "com.apple.dock");
        assert_eq!(setting.key, "tilesize");
        assert_eq!(setting.original_value, Some("36".to_string()));
        assert_eq!(setting.new_value, "46");

        // Test external command state
        let command = ExternalCommandState {
            cmd: "echo".to_string(),
            args: vec!["Hello".to_string(), "World".to_string()],
            sudo: false,
        };
        assert_eq!(command.cmd, "echo");
        assert_eq!(command.args, vec!["Hello".to_string(), "World".to_string()]);
        assert_eq!(command.sudo, false);
    }

    #[test]
    fn test_snapshot_serialization() {
        // Create a comprehensive snapshot with test data
        let mut snapshot = Snapshot::new();

        // Add multiple settings with different patterns
        snapshot.settings.push(SettingState {
            domain: "com.apple.dock".to_string(),
            key: "tilesize".to_string(),
            original_value: Some("36".to_string()),
            new_value: "46".to_string(),
        });

        snapshot.settings.push(SettingState {
            domain: "com.apple.finder".to_string(),
            key: "ShowPathbar".to_string(),
            original_value: None, // Test null original value
            new_value: "1".to_string(),
        });

        snapshot.settings.push(SettingState {
            domain: "NSGlobalDomain".to_string(),
            key: "ApplePressAndHoldEnabled".to_string(),
            original_value: Some("0".to_string()),
            new_value: "1".to_string(),
        });

        // Add multiple external commands
        snapshot.external_commands.push(ExternalCommandState {
            cmd: "echo".to_string(),
            args: vec!["Hello".to_string()],
            sudo: false,
        });

        snapshot.external_commands.push(ExternalCommandState {
            cmd: "hostname".to_string(),
            args: vec!["-s".to_string(), "macbook".to_string()],
            sudo: true,
        });

        // Create a temporary file to store the snapshot
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.json");

        // Save the snapshot
        snapshot.save_to_file(&snapshot_path).unwrap();

        // Verify file exists and has content
        assert!(snapshot_path.exists());
        let content = fs::read_to_string(&snapshot_path).unwrap();
        assert!(content.contains("com.apple.dock"));
        assert!(content.contains("tilesize"));

        // Load the snapshot back
        let loaded_snapshot = Snapshot::load_from_file(&snapshot_path).unwrap();

        // Verify contents match
        assert_eq!(loaded_snapshot.settings.len(), 3);
        assert_eq!(loaded_snapshot.external_commands.len(), 2);

        // Convert to HashMap for easier testing
        let settings_map: HashMap<_, _> = loaded_snapshot
            .settings
            .iter()
            .map(|s| ((s.domain.clone(), s.key.clone()), s))
            .collect();

        // Check dock setting
        let dock_setting = settings_map
            .get(&("com.apple.dock".to_string(), "tilesize".to_string()))
            .unwrap();
        assert_eq!(dock_setting.original_value, Some("36".to_string()));
        assert_eq!(dock_setting.new_value, "46");

        // Check finder setting (null original)
        let finder_setting = settings_map
            .get(&("com.apple.finder".to_string(), "ShowPathbar".to_string()))
            .unwrap();
        assert_eq!(finder_setting.original_value, None);
        assert_eq!(finder_setting.new_value, "1");

        // Check global setting
        let global_setting = settings_map
            .get(&(
                "NSGlobalDomain".to_string(),
                "ApplePressAndHoldEnabled".to_string(),
            ))
            .unwrap();
        assert_eq!(global_setting.original_value, Some("0".to_string()));
        assert_eq!(global_setting.new_value, "1");

        // Check external commands
        let echo_cmd = &loaded_snapshot.external_commands[0];
        assert_eq!(echo_cmd.cmd, "echo");
        assert_eq!(echo_cmd.args, vec!["Hello".to_string()]);
        assert_eq!(echo_cmd.sudo, false);

        let hostname_cmd = &loaded_snapshot.external_commands[1];
        assert_eq!(hostname_cmd.cmd, "hostname");
        assert_eq!(
            hostname_cmd.args,
            vec!["-s".to_string(), "macbook".to_string()]
        );
        assert_eq!(hostname_cmd.sudo, true);
    }

    #[test]
    fn test_snapshot_error_handling() {
        // Test loading from non-existent file
        let result = Snapshot::load_from_file(&PathBuf::from("/nonexistent/path"));
        assert!(result.is_err());

        // Test loading from invalid JSON
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid.json");
        fs::write(&invalid_path, "this is not valid json").unwrap();

        let result = Snapshot::load_from_file(&invalid_path);
        assert!(result.is_err());

        // Test writing to invalid path
        let snapshot = Snapshot::new();
        let invalid_dir = PathBuf::from("/nonexistent/directory/snapshot.json");
        let result = snapshot.save_to_file(&invalid_dir);
        assert!(result.is_err());
    }
}
