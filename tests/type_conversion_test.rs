// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use cutler::domains::convert::{
        prefvalue_to_toml, toml_edit_to_prefvalue, toml_edit_to_toml, toml_to_prefvalue,
    };
    use defaults_rs::PrefValue;
    use toml::Value;

    #[test]
    fn test_toml_to_prefvalue_basic_types() {
        // String
        let s = Value::String("hello".to_string());
        let pref = toml_to_prefvalue(&s).unwrap();
        match pref {
            PrefValue::String(val) => assert_eq!(val, "hello"),
            _ => panic!("Expected String"),
        }

        // Integer
        let i = Value::Integer(42);
        let pref = toml_to_prefvalue(&i).unwrap();
        match pref {
            PrefValue::Integer(val) => assert_eq!(val, 42),
            _ => panic!("Expected Integer"),
        }

        // Float
        let f = Value::Float(PI);
        let pref = toml_to_prefvalue(&f).unwrap();
        match pref {
            PrefValue::Float(val) => assert!((val - PI).abs() < 0.001),
            _ => panic!("Expected Float"),
        }

        // Boolean
        let b = Value::Boolean(true);
        let pref = toml_to_prefvalue(&b).unwrap();
        match pref {
            PrefValue::Boolean(val) => assert!(val),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn test_toml_to_prefvalue_array() {
        let arr = Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);

        let pref = toml_to_prefvalue(&arr).unwrap();
        match pref {
            PrefValue::Array(vals) => {
                assert_eq!(vals.len(), 3);
                match &vals[0] {
                    PrefValue::Integer(v) => assert_eq!(*v, 1),
                    _ => panic!("Expected Integer"),
                }
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_toml_to_prefvalue_dictionary() {
        let mut map = toml::map::Map::new();
        map.insert("key1".to_string(), Value::String("value1".to_string()));
        map.insert("key2".to_string(), Value::Integer(42));
        let table = Value::Table(map);

        let pref = toml_to_prefvalue(&table).unwrap();
        match pref {
            PrefValue::Dictionary(dict) => {
                assert_eq!(dict.len(), 2);
                match dict.get("key1").unwrap() {
                    PrefValue::String(s) => assert_eq!(s, "value1"),
                    _ => panic!("Expected String"),
                }
                match dict.get("key2").unwrap() {
                    PrefValue::Integer(i) => assert_eq!(*i, 42),
                    _ => panic!("Expected Integer"),
                }
            }
            _ => panic!("Expected Dictionary"),
        }
    }

    #[test]
    fn test_prefvalue_to_toml_roundtrip() {
        // Test that we can convert back and forth
        let original = Value::Array(vec![
            Value::String("test".to_string()),
            Value::Integer(42),
            Value::Boolean(true),
        ]);

        let pref = toml_to_prefvalue(&original).unwrap();
        let back = prefvalue_to_toml(&pref).unwrap();

        assert_eq!(original, back);
    }

    #[test]
    fn test_toml_edit_to_toml_inline_table() {
        // Parse an inline table with toml_edit
        let toml_str = r#"
test = { key1 = "value1", key2 = 42 }
"#;
        let doc: toml_edit::DocumentMut = toml_str.parse().unwrap();
        let value = doc.get("test").unwrap();

        if let toml_edit::Item::Value(v) = value {
            let toml_val = toml_edit_to_toml(v).unwrap();

            // Should be a table
            assert!(toml_val.is_table());
            let table = toml_val.as_table().unwrap();
            assert_eq!(table.get("key1").unwrap().as_str().unwrap(), "value1");
            assert_eq!(table.get("key2").unwrap().as_integer().unwrap(), 42);
        } else {
            panic!("Expected Value");
        }
    }

    #[test]
    fn test_toml_edit_to_prefvalue_inline_table() {
        // Parse an inline table with toml_edit and convert to PrefValue
        let toml_str = r#"
test = { Preview = false, MetaData = true }
"#;
        let doc: toml_edit::DocumentMut = toml_str.parse().unwrap();
        let value = doc.get("test").unwrap();

        if let toml_edit::Item::Value(v) = value {
            let pref_val = toml_edit_to_prefvalue(v).unwrap();

            // Should be a dictionary
            match pref_val {
                PrefValue::Dictionary(dict) => {
                    assert_eq!(dict.len(), 2);
                    match dict.get("Preview").unwrap() {
                        PrefValue::Boolean(b) => assert!(!b),
                        _ => panic!("Expected Boolean"),
                    }
                    match dict.get("MetaData").unwrap() {
                        PrefValue::Boolean(b) => assert!(b),
                        _ => panic!("Expected Boolean"),
                    }
                }
                _ => panic!("Expected Dictionary"),
            }
        } else {
            panic!("Expected Value");
        }
    }
}
