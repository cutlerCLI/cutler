// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(test)]
mod tests {
    use cutler::config::core::Config;
    use cutler::domains::collect;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_inline_table_as_dictionary_value() {
        // Test that inline tables are treated as dictionary values, not domains
        let config_content = r#"
[set.finder]
FXInfoPanesExpanded = { Preview = false, MetaData = true }
ShowPathbar = true
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config: Config = toml::from_str(config_content).unwrap();
        let mut config_with_path = config;
        config_with_path.path = temp_file.path().to_path_buf();

        let domains = collect(&config_with_path).await.unwrap();

        // Should only have "finder" domain, not "finder.FXInfoPanesExpanded"
        assert_eq!(domains.len(), 1);
        assert!(domains.contains_key("finder"));
        assert!(!domains.contains_key("finder.FXInfoPanesExpanded"));

        let finder = domains.get("finder").unwrap();

        // FXInfoPanesExpanded should be a table value
        assert!(finder.contains_key("FXInfoPanesExpanded"));
        let fx_info = finder.get("FXInfoPanesExpanded").unwrap();
        assert!(fx_info.is_table());

        let fx_table = fx_info.as_table().unwrap();
        assert!(!fx_table.get("Preview").unwrap().as_bool().unwrap());
        assert!(fx_table.get("MetaData").unwrap().as_bool().unwrap());

        // ShowPathbar should be a boolean
        assert!(finder.get("ShowPathbar").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_nested_table_header_as_domain() {
        // Test that nested table headers create new domains
        let config_content = r#"
[set.dock]
tilesize = 50

[set.NSGlobalDomain.com.apple.keyboard]
fnState = false
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config: Config = toml::from_str(config_content).unwrap();
        let mut config_with_path = config;
        config_with_path.path = temp_file.path().to_path_buf();

        let domains = collect(&config_with_path).await.unwrap();

        // Should have "dock" and "NSGlobalDomain.com.apple.keyboard"
        assert_eq!(domains.len(), 2);
        assert!(domains.contains_key("dock"));
        assert!(domains.contains_key("NSGlobalDomain.com.apple.keyboard"));

        let dock = domains.get("dock").unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_integer().unwrap(), 50);

        let keyboard = domains.get("NSGlobalDomain.com.apple.keyboard").unwrap();
        assert!(!keyboard.get("fnState").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_complex_types() {
        // Test various complex types
        let config_content = r#"
[set.dock]
tilesize = 50
autohide = true

[set.NSGlobalDomain]
"com.apple.dock.fnState" = false
exampleArrayOfInts = [1, 2, 3]
exampleArrayOfStrings = ["one", "two", "three"]

[set.finder]
FXInfoPanesExpanded = { Preview = false, MetaData = true, Comments = false }
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config: Config = toml::from_str(config_content).unwrap();
        let mut config_with_path = config;
        config_with_path.path = temp_file.path().to_path_buf();

        let domains = collect(&config_with_path).await.unwrap();

        assert_eq!(domains.len(), 3);

        // Check dock settings
        let dock = domains.get("dock").unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_integer().unwrap(), 50);
        assert!(dock.get("autohide").unwrap().as_bool().unwrap());

        // Check NSGlobalDomain settings
        let global = domains.get("NSGlobalDomain").unwrap();
        assert!(
            !global
                .get("com.apple.dock.fnState")
                .unwrap()
                .as_bool()
                .unwrap()
        );

        let int_array = global
            .get("exampleArrayOfInts")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(int_array.len(), 3);
        assert_eq!(int_array[0].as_integer().unwrap(), 1);
        assert_eq!(int_array[1].as_integer().unwrap(), 2);
        assert_eq!(int_array[2].as_integer().unwrap(), 3);

        let str_array = global
            .get("exampleArrayOfStrings")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(str_array.len(), 3);
        assert_eq!(str_array[0].as_str().unwrap(), "one");
        assert_eq!(str_array[1].as_str().unwrap(), "two");
        assert_eq!(str_array[2].as_str().unwrap(), "three");

        // Check finder inline table
        let finder = domains.get("finder").unwrap();
        let fx_table = finder
            .get("FXInfoPanesExpanded")
            .unwrap()
            .as_table()
            .unwrap();
        assert!(!fx_table.get("Preview").unwrap().as_bool().unwrap());
        assert!(fx_table.get("MetaData").unwrap().as_bool().unwrap());
        assert!(!fx_table.get("Comments").unwrap().as_bool().unwrap());
    }
}
