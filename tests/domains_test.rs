// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use cutler::config::core::Config;
    use cutler::domains::{collect, effective};
    use std::collections::HashMap;
    use toml::{Value, value::Table};

    fn config_with_set(set: HashMap<String, HashMap<String, Value>>) -> Config {
        Config {
            lock: None,
            set: Some(set),
            vars: None,
            command: None,
            brew: None,
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
        assert_eq!(domains.len(), 1);
        let got = domains.get("root.nested").unwrap();
        assert_eq!(
            got.get("inner_key").unwrap().as_str().unwrap(),
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

        let domains = collect(&parsed).unwrap();
        assert_eq!(domains.len(), 2);
        let dock = domains.get("dock").unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_str().unwrap(), "50");
        assert!(dock.get("autohide").unwrap().as_bool().unwrap());
        let kb = domains.get("NSGlobalDomain.com.apple.keyboard").unwrap();
        assert!(!kb.get("fnState").unwrap().as_bool().unwrap());
    }
}
