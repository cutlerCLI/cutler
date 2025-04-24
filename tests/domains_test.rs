#[cfg(test)]
mod tests {
    use cutler::domains::{collect, effective, needs_prefix};
    use toml::{Value, value::Table};

    #[test]
    fn test_collect_domains_simple() {
        // [domain]
        //   key1 = "value1"
        let mut table = Table::new();
        table.insert("key1".into(), Value::String("value1".into()));
        let mut root = Table::new();
        root.insert("domain".into(), Value::Table(table));
        let parsed = Value::Table(root);

        let domains = collect(&parsed).unwrap();
        assert_eq!(domains.len(), 1);
        let got = domains.get("domain").unwrap();
        assert_eq!(got.get("key1").unwrap().as_str().unwrap(), "value1");
    }

    #[test]
    fn test_collect_domains_nested() {
        // [root.nested]
        //   inner_key = "inner_value"
        let mut inner = Table::new();
        inner.insert("inner_key".into(), Value::String("inner_value".into()));
        let mut nested = Table::new();
        nested.insert("nested".into(), Value::Table(inner));
        let mut root = Table::new();
        root.insert("root".into(), Value::Table(nested));

        let domains = collect(&Value::Table(root)).unwrap();
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
    fn test_needs_prefix() {
        assert!(needs_prefix("dock"));
        assert!(needs_prefix("finder"));
        assert!(!needs_prefix("NSGlobalDomain"));
        assert!(!needs_prefix("NSGlobalDomain.x"));
    }
}
