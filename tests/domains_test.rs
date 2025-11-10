// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(test)]
mod tests {
    use cutler::config::core::Config;
    use cutler::domains::{collect, effective};
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use toml::{Value, value::Table};

    fn config_with_set(set: HashMap<String, HashMap<String, Value>>) -> Config {
        Config {
            lock: None,
            set: Some(set),
            vars: None,
            command: None,
            brew: None,
            mas: None,
            remote: None,
            path: Default::default(),
        }
    }

    #[test]
    fn test_collect_domains_simple() {
        // [set.domain]
        //   key1 = "value1"
        let mut domain_map = HashMap::new();
        domain_map.insert("key1".into(), Value::String("value1".into()));
        let mut set_map = HashMap::new();
        set_map.insert("domain".into(), domain_map);

        let config = config_with_set(set_map);

        let domains = collect(&config).unwrap();
        assert_eq!(domains.len(), 1);
        let got = domains.get("domain").unwrap();
        assert_eq!(got.get("key1").unwrap().as_str().unwrap(), "value1");
    }

    #[test]
    fn test_collect_domains_nested() {
        // [set.root]
        //   [set.root.nested]
        //   inner_key = "inner_value"
        //
        // This test now tests that we DON'T flatten nested Value::Tables
        // when they're created programmatically (no file path).
        // Instead, they should be treated as inline table values.
        let mut inner: HashMap<String, Value> = HashMap::new();
        inner.insert("inner_key".into(), Value::String("inner_value".into()));
        let mut nested = HashMap::new();
        nested.insert(
            "nested".into(),
            Value::Table({
                let mut tbl = Table::new();
                tbl.insert("inner_key".into(), Value::String("inner_value".into()));
                tbl
            }),
        );
        let mut set_map = HashMap::new();
        set_map.insert("root".into(), nested);

        let config = config_with_set(set_map);

        let domains = collect(&config).unwrap();
        // With the new behavior, "nested" is treated as an inline table value
        // since we don't have a file path to parse with toml_edit
        assert_eq!(domains.len(), 1);
        let got = domains.get("root").unwrap();

        // "nested" should be a table value, not a flattened domain
        assert!(got.contains_key("nested"));
        let nested_val = got.get("nested").unwrap();
        assert!(nested_val.is_table());
        let nested_table = nested_val.as_table().unwrap();
        assert_eq!(
            nested_table.get("inner_key").unwrap().as_str().unwrap(),
            "inner_value"
        );
    }

    #[test]
    fn test_get_effective_domain_and_key() {
        let (d, k) = effective("finder", "ShowPathbar");
        assert_eq!((d, k), ("com.apple.finder".into(), "ShowPathbar".into()));

        let (d, k) = effective("NSGlobalDomain", "Foo");
        assert_eq!((d, k), ("NSGlobalDomain".into(), "Foo".into()));

        let (d, k) = effective("NSGlobalDomain.bar", "Baz");
        assert_eq!((d, k), ("NSGlobalDomain".into(), "bar.Baz".into()));
    }

    #[test]
    fn test_collect_domains_set() {
        let config_content = r#"
[set.dock]
tilesize = "50"
autohide = true

[set.NSGlobalDomain.com.apple.keyboard]
fnState = false
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let parsed: Config = toml::from_str(config_content).unwrap();
        let mut config_with_path = parsed;
        config_with_path.path = temp_file.path().to_path_buf();

        let domains = collect(&config_with_path).unwrap();
        assert_eq!(domains.len(), 2);
        let dock = domains.get("dock").unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_str().unwrap(), "50");
        assert!(dock.get("autohide").unwrap().as_bool().unwrap());
        let kb = domains.get("NSGlobalDomain.com.apple.keyboard").unwrap();
        assert!(!kb.get("fnState").unwrap().as_bool().unwrap());
    }
}
