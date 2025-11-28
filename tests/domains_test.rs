// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(test)]
mod tests {
    use cutler::config::Config;
    use cutler::domains::{collect, effective};
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a Config pointing to a temp file with given TOML content.
    fn config_from_toml(content: &str) -> (NamedTempFile, Config) {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        let config = Config::new(temp_file.path().to_path_buf());
        (temp_file, config)
    }

    #[tokio::test]
    async fn test_collect_domains_simple() {
        // [set.domain]
        //   key1 = "value1"
        let config_content = r#"
[set.domain]
key1 = "value1"
"#;
        let (_temp_file, config) = config_from_toml(config_content);

        let domains = collect(&config).await.unwrap();
        assert_eq!(domains.len(), 1);
        let got = domains.get("domain").unwrap();
        assert_eq!(got.get("key1").unwrap().as_str().unwrap(), "value1");
    }

    #[tokio::test]
    async fn test_collect_domains_nested() {
        // [set.root]
        //   [set.root.nested]
        //   inner_key = "inner_value"
        //
        // This tests that nested table headers are treated as separate domains.
        let config_content = r#"
[set.root]
top_key = "top_value"

[set.root.nested]
inner_key = "inner_value"
"#;
        let (_temp_file, config) = config_from_toml(config_content);

        let domains = collect(&config).await.unwrap();
        // With the new behavior, "root.nested" becomes a separate domain
        assert_eq!(domains.len(), 2);

        let root = domains.get("root").unwrap();
        assert!(root.contains_key("top_key"));
        assert_eq!(root.get("top_key").unwrap().as_str().unwrap(), "top_value");

        let nested = domains.get("root.nested").unwrap();
        assert!(nested.contains_key("inner_key"));
        assert_eq!(
            nested.get("inner_key").unwrap().as_str().unwrap(),
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

    #[tokio::test]
    async fn test_collect_domains_set() {
        let config_content = r#"
[set.dock]
tilesize = "50"
autohide = true

[set.NSGlobalDomain.com.apple.keyboard]
fnState = false
"#;
        let (_temp_file, config) = config_from_toml(config_content);

        let domains = collect(&config).await.unwrap();
        assert_eq!(domains.len(), 2);
        let dock = domains.get("dock").unwrap();
        assert_eq!(dock.get("tilesize").unwrap().as_str().unwrap(), "50");
        assert!(dock.get("autohide").unwrap().as_bool().unwrap());
        let kb = domains.get("NSGlobalDomain.com.apple.keyboard").unwrap();
        assert!(!kb.get("fnState").unwrap().as_bool().unwrap());
    }
}
