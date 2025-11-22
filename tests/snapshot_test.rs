// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(test)]
mod tests {
    use cutler::{
        config::get_config_path,
        domains::convert::SerializablePrefValue,
        snapshot::{
            core::{SettingState, Snapshot},
            get_snapshot_path,
        },
    };
    use std::{collections::HashMap, env, path::PathBuf};
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_get_snapshot_path() {
        // Test that get_snapshot_path returns .cutler_snapshot in the home directory
        let snapshot_path = get_snapshot_path().await.unwrap();
        assert_eq!(
            snapshot_path,
            get_config_path()
                .await
                .unwrap()
                .parent()
                .unwrap()
                .join("snapshot.json")
        );
    }

    #[tokio::test]
    async fn test_snapshot_basic() {
        // Test creation
        let snapshot = Snapshot::new().await;
        assert_eq!(snapshot.settings.len(), 0);
        assert_eq!(snapshot.exec_run_count, 0);
        assert_eq!(snapshot.version, env!("CARGO_PKG_VERSION"));

        // Test setting state
        let setting = SettingState {
            domain: "com.apple.dock".to_string(),
            key: "tilesize".to_string(),
            original_value: Some(SerializablePrefValue::Integer(36)),
        };
        assert_eq!(setting.domain, "com.apple.dock");
        assert_eq!(setting.key, "tilesize");
        assert_eq!(
            setting.original_value,
            Some(SerializablePrefValue::Integer(36))
        );
    }

    #[tokio::test]
    async fn test_snapshot_serialization() {
        // Create a comprehensive snapshot with test data
        let mut snapshot = Snapshot::new().await;

        // Add multiple settings with different patterns
        snapshot.settings.push(SettingState {
            domain: "com.apple.dock".to_string(),
            key: "tilesize".to_string(),
            original_value: Some(SerializablePrefValue::Integer(36)),
        });

        snapshot.settings.push(SettingState {
            domain: "com.apple.finder".to_string(),
            key: "ShowPathbar".to_string(),
            original_value: None, // Test null original value
        });

        snapshot.settings.push(SettingState {
            domain: "NSGlobalDomain".to_string(),
            key: "ApplePressAndHoldEnabled".to_string(),
            original_value: Some(SerializablePrefValue::Boolean(false)),
        });

        // Create a temporary file to store the snapshot
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.json");

        // Save the snapshot
        snapshot.path = snapshot_path.clone();
        snapshot.save().await.unwrap();

        // Verify file exists and has content
        assert!(fs::try_exists(&snapshot_path).await.unwrap());
        let content = fs::read_to_string(&snapshot_path).await.unwrap();
        assert!(content.contains("com.apple.dock"));
        assert!(content.contains("tilesize"));

        // Load the snapshot back
        let loaded_snapshot = Snapshot::load(&snapshot_path).await.unwrap();

        // Verify contents match
        assert_eq!(loaded_snapshot.settings.len(), 3);

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
        assert_eq!(
            dock_setting.original_value,
            Some(SerializablePrefValue::Integer(36))
        );

        // Check finder setting (null original)
        let finder_setting = settings_map
            .get(&("com.apple.finder".to_string(), "ShowPathbar".to_string()))
            .unwrap();
        assert_eq!(finder_setting.original_value, None);

        // Check global setting
        let global_setting = settings_map
            .get(&(
                "NSGlobalDomain".to_string(),
                "ApplePressAndHoldEnabled".to_string(),
            ))
            .unwrap();
        assert_eq!(
            global_setting.original_value,
            Some(SerializablePrefValue::Boolean(false))
        );
    }

    #[tokio::test]
    async fn test_snapshot_error_handling() {
        // Test loading from non-existent file
        let result = Snapshot::load(&PathBuf::from("/nonexistent/path")).await;
        assert!(result.is_err());

        // Test loading from invalid JSON
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid.json");
        fs::write(&invalid_path, "this is not valid json")
            .await
            .unwrap();

        let result = Snapshot::load(&invalid_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_snapshot_complex_types() {
        use std::collections::HashMap;

        // Test snapshot with complex types (arrays, dictionaries)
        let mut snapshot = Snapshot::new().await;

        // Array type
        snapshot.settings.push(SettingState {
            domain: "NSGlobalDomain".to_string(),
            key: "exampleArray".to_string(),
            original_value: Some(SerializablePrefValue::Array(vec![
                SerializablePrefValue::Integer(1),
                SerializablePrefValue::Integer(2),
                SerializablePrefValue::Integer(3),
            ])),
        });

        // Dictionary type
        let mut dict = HashMap::new();
        dict.insert("Preview".to_string(), SerializablePrefValue::Boolean(false));
        dict.insert("MetaData".to_string(), SerializablePrefValue::Boolean(true));

        snapshot.settings.push(SettingState {
            domain: "com.apple.finder".to_string(),
            key: "FXInfoPanesExpanded".to_string(),
            original_value: Some(SerializablePrefValue::Dictionary(dict)),
        });

        // Create a temporary file to store the snapshot
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_complex_snapshot.json");

        // Save the snapshot
        snapshot.path = snapshot_path.clone();
        snapshot.save().await.unwrap();

        // Load the snapshot back
        let loaded_snapshot = Snapshot::load(&snapshot_path).await.unwrap();

        // Verify array
        let array_setting = &loaded_snapshot.settings[0];
        assert_eq!(array_setting.domain, "NSGlobalDomain");
        assert_eq!(array_setting.key, "exampleArray");
        match &array_setting.original_value {
            Some(SerializablePrefValue::Array(arr)) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], SerializablePrefValue::Integer(1));
                assert_eq!(arr[1], SerializablePrefValue::Integer(2));
                assert_eq!(arr[2], SerializablePrefValue::Integer(3));
            }
            _ => panic!("Expected array type"),
        }

        // Verify dictionary
        let dict_setting = &loaded_snapshot.settings[1];
        assert_eq!(dict_setting.domain, "com.apple.finder");
        assert_eq!(dict_setting.key, "FXInfoPanesExpanded");
        match &dict_setting.original_value {
            Some(SerializablePrefValue::Dictionary(dict)) => {
                assert_eq!(dict.len(), 2);
                assert_eq!(
                    dict.get("Preview"),
                    Some(&SerializablePrefValue::Boolean(false))
                );
                assert_eq!(
                    dict.get("MetaData"),
                    Some(&SerializablePrefValue::Boolean(true))
                );
            }
            _ => panic!("Expected dictionary type"),
        }
    }
}
