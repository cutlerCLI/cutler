#[cfg(test)]
mod tests {
    use cutler::domains::{
        collect_domains, flatten_domains, get_effective_domain, get_effective_domain_and_key,
        needs_prefix,
    };
    use toml::Value;
    use toml::value::Table;

    #[test]
    fn test_flatten_domains_simple() {
        let mut input = Table::new();
        input.insert("key1".to_string(), Value::String("value1".to_string()));

        let mut result = Vec::new();
        flatten_domains(Some("domain".to_string()), &input, &mut result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "domain");
        assert_eq!(result[0].1.get("key1").unwrap().as_str().unwrap(), "value1");
    }

    #[test]
    fn test_flatten_domains_nested() {
        let mut inner = Table::new();
        inner.insert(
            "inner_key".to_string(),
            Value::String("inner_value".to_string()),
        );

        let mut input = Table::new();
        input.insert("nested".to_string(), Value::Table(inner));

        let mut result = Vec::new();
        flatten_domains(Some("root".to_string()), &input, &mut result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "root.nested");
        assert_eq!(
            result[0].1.get("inner_key").unwrap().as_str().unwrap(),
            "inner_value"
        );
    }

    #[test]
    fn test_get_effective_domain_and_key() {
        // Test standard domain
        let (domain, key) = get_effective_domain_and_key("finder", "ShowPathbar");
        assert_eq!(domain, "com.apple.finder");
        assert_eq!(key, "ShowPathbar");

        // Test NSGlobalDomain
        let (domain, key) =
            get_effective_domain_and_key("NSGlobalDomain", "ApplePressAndHoldEnabled");
        assert_eq!(domain, "NSGlobalDomain");
        assert_eq!(key, "ApplePressAndHoldEnabled");

        // Test NSGlobalDomain with nested subdomains
        let (domain, key) =
            get_effective_domain_and_key("NSGlobalDomain.com.apple.keyboard", "fnState");
        assert_eq!(domain, "NSGlobalDomain");
        assert_eq!(key, "com.apple.keyboard.fnState");
    }

    #[test]
    fn test_get_effective_domain() {
        assert_eq!(get_effective_domain("dock"), "com.apple.dock");
        assert_eq!(get_effective_domain("NSGlobalDomain"), "NSGlobalDomain");
        assert_eq!(
            get_effective_domain("NSGlobalDomain.something"),
            "NSGlobalDomain"
        );
    }

    #[test]
    fn test_needs_prefix() {
        assert!(needs_prefix("dock"));
        assert!(needs_prefix("finder"));
        assert!(!needs_prefix("NSGlobalDomain"));
        assert!(!needs_prefix("NSGlobalDomain.something"));
    }

    #[test]
    fn test_collect_domains() {
        // Create a test TOML structure
        let mut dock_table = Table::new();
        dock_table.insert("tilesize".to_string(), Value::Integer(46));
        dock_table.insert("autohide".to_string(), Value::Boolean(true));

        let mut finder_table = Table::new();
        finder_table.insert("ShowPathbar".to_string(), Value::Boolean(true));

        let mut global_table = Table::new();
        global_table.insert("ApplePressAndHoldEnabled".to_string(), Value::Boolean(true));

        let mut root = Table::new();
        root.insert("dock".to_string(), Value::Table(dock_table));
        root.insert("finder".to_string(), Value::Table(finder_table));
        root.insert("NSGlobalDomain".to_string(), Value::Table(global_table));

        let parsed = Value::Table(root);

        let domains = collect_domains(&parsed).unwrap();

        assert_eq!(domains.len(), 3);
        assert!(domains.contains_key("dock"));
        assert!(domains.contains_key("finder"));
        assert!(domains.contains_key("NSGlobalDomain"));

        let dock_settings = domains.get("dock").unwrap();
        assert_eq!(
            dock_settings.get("tilesize").unwrap().as_integer().unwrap(),
            46
        );
    }
}
