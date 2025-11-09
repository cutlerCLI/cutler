// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(test)]
mod tests {
    use cutler::config::core::Config;
    use cutler::domains::{collect, effective};
    use cutler::domains::convert::{toml_to_prefvalue, prefvalue_to_toml};
    use std::collections::HashMap;
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

    #[tokio::test]
    async fn test_collect_domains_simple() {
        // [set.domain]
        //   key1 = "value1"
        let mut domain_map = HashMap::new();
        domain_map.insert("key1".into(), Value::String("value1".into()));
        let mut set_map = HashMap::new();
        set_map.insert("domain".into(), domain_map);

        let config = config_with_set(set_map);

        let domains = collect(&config).await.unwrap();
        assert_eq!(domains.len(), 1);
        let got = domains.get("domain").unwrap();
        assert_eq!(got.get("key1").unwrap().as_str().unwrap(), "value1");
    }

    #[tokio::test]
    async fn test_collect_domains_nested() {
        // [set.root.nested]
        //   inner_key = "inner_value"
        // When TOML creates nested structures, flatten_domains processes them
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

        let domains = collect(&config).await.unwrap();
        assert_eq!(domains.len(), 1);
        let got = domains.get("root.nested").unwrap();
        assert_eq!(
            got.get("inner_key").unwrap().as_str().unwrap(),
            "inner_value"
        );
    }

    #[tokio::test]
    async fn test_get_effective_domain_and_key() {
        let (d, k) = effective("finder", "ShowPathbar");
        assert_eq!((d, k), ("com.apple.finder".into(), "ShowPathbar".into()));

        let (d, k) = effective("NSGlobalDomain", "Foo");
        assert_eq!((d, k), ("NSGlobalDomain".into(), "Foo".into()));

        let (d, k) = effective("NSGlobalDomain.bar", "Baz");
        assert_eq!((d, k), ("NSGlobalDomain".into(), "bar.Baz".into()));
    }

    #[tokio::test]
    async fn test_collect_domains_set() {
        let parsed: Config = toml::from_str(
            r#"
[set.dock]
tilesize = "50"
autohide = true

[set.NSGlobalDomain.com.apple.keyboard]
fnState = false
"#,
        )
        .unwrap();

        let domains = collect(&parsed).await.unwrap();
        assert_eq!(domains.len(), 2);
        let dock = domains.get("dock").unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_str().unwrap(), "50");
        assert!(dock.get("autohide").unwrap().as_bool().unwrap());
        let kb = domains.get("NSGlobalDomain.com.apple.keyboard").unwrap();
        assert!(!kb.get("fnState").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_toml_to_prefvalue_array() {
        // Test array conversion
        let toml_array = Value::Array(vec![
            Value::String("item1".to_string()),
            Value::String("item2".to_string()),
            Value::Integer(42),
        ]);

        let pref_value = toml_to_prefvalue(&toml_array).unwrap();
        
        // Convert back to TOML to verify round-trip
        let back_to_toml = prefvalue_to_toml(&pref_value);
        assert_eq!(back_to_toml, toml_array);
    }

    #[tokio::test]
    async fn test_toml_to_prefvalue_dictionary() {
        // Test dictionary conversion
        let mut tbl = Table::new();
        tbl.insert("key1".to_string(), Value::String("value1".to_string()));
        tbl.insert("key2".to_string(), Value::Integer(100));
        tbl.insert("key3".to_string(), Value::Boolean(true));
        
        let toml_dict = Value::Table(tbl);
        let pref_value = toml_to_prefvalue(&toml_dict).unwrap();
        
        // Convert back to TOML to verify round-trip
        let back_to_toml = prefvalue_to_toml(&pref_value);
        assert_eq!(back_to_toml, toml_dict);
    }

    #[tokio::test]
    async fn test_toml_to_prefvalue_nested() {
        // Test nested structures
        let mut inner_tbl = Table::new();
        inner_tbl.insert("nested_key".to_string(), Value::String("nested_value".to_string()));
        
        let mut outer_tbl = Table::new();
        outer_tbl.insert("outer_key".to_string(), Value::Table(inner_tbl));
        outer_tbl.insert("array_key".to_string(), Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        
        let toml_nested = Value::Table(outer_tbl);
        let pref_value = toml_to_prefvalue(&toml_nested).unwrap();
        
        // Convert back to TOML to verify round-trip
        let back_to_toml = prefvalue_to_toml(&pref_value);
        assert_eq!(back_to_toml, toml_nested);
    }

    #[tokio::test]
    async fn test_collect_domains_with_arrays() {
        // Test collecting domains with array values
        let parsed: Config = toml::from_str(
            r#"
[set.test]
simple_array = ["item1", "item2", "item3"]
mixed_array = [1, 2, 3]
"#,
        )
        .unwrap();

        let domains = collect(&parsed).await.unwrap();
        assert_eq!(domains.len(), 1);
        let test_domain = domains.get("test").unwrap();
        
        let simple_array = test_domain.get("simple_array").unwrap();
        assert!(simple_array.is_array());
        let arr = simple_array.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        
        let mixed_array = test_domain.get("mixed_array").unwrap();
        assert!(mixed_array.is_array());
    }

    #[tokio::test]
    async fn test_collect_domains_with_inline_tables() {
        // Test that inline tables (dictionaries) are kept as values, not flattened
        let parsed: Config = toml::from_str(
            r#"
[set.finder]
FXInfoPanelsExpanded = { Preview = false, General = true }
ShowPathbar = true
"#,
        )
        .unwrap();

        let domains = collect(&parsed).await.unwrap();
        // Should only have "finder" domain, not "finder.FXInfoPanelsExpanded"
        assert_eq!(domains.len(), 1);
        assert!(domains.contains_key("finder"));
        assert!(!domains.contains_key("finder.FXInfoPanelsExpanded"));
        assert!(!domains.contains_key("finder.Preview"));
        assert!(!domains.contains_key("finder.General"));
        
        let finder_domain = domains.get("finder").unwrap();
        
        // FXInfoPanelsExpanded should be a table value
        let fx_info = finder_domain.get("FXInfoPanelsExpanded").unwrap();
        assert!(fx_info.is_table());
        
        let fx_table = fx_info.as_table().unwrap();
        assert_eq!(fx_table.get("Preview").unwrap().as_bool().unwrap(), false);
        assert_eq!(fx_table.get("General").unwrap().as_bool().unwrap(), true);
        
        // ShowPathbar should still be there
        assert_eq!(finder_domain.get("ShowPathbar").unwrap().as_bool().unwrap(), true);
    }

    #[tokio::test]
    async fn test_collect_menuextra_clock_with_inline_table() {
        // Test the specific case from the user: [set.menuextra.clock] with inline table values
        let parsed: Config = toml::from_str(
            r#"
[set.menuextra.clock]
key1 = 1
key2 = { part1 = 1, part2 = 2 }
"#,
        )
        .unwrap();

        let domains = collect(&parsed).await.unwrap();
        // Should have "menuextra.clock" domain (from section header flattening)
        // but NOT flatten the inline table key2
        assert_eq!(domains.len(), 1);
        assert!(domains.contains_key("menuextra.clock"));
        assert!(!domains.contains_key("menuextra.clock.key2"));
        assert!(!domains.contains_key("menuextra.clock.part1"));
        
        let clock_domain = domains.get("menuextra.clock").unwrap();
        
        // key1 should be an integer
        assert_eq!(clock_domain.get("key1").unwrap().as_integer().unwrap(), 1);
        
        // key2 should be a table (dictionary) value
        let key2 = clock_domain.get("key2").unwrap();
        assert!(key2.is_table());
        
        let key2_table = key2.as_table().unwrap();
        assert_eq!(key2_table.get("part1").unwrap().as_integer().unwrap(), 1);
        assert_eq!(key2_table.get("part2").unwrap().as_integer().unwrap(), 2);
    }
}
