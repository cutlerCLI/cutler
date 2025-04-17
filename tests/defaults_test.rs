#[cfg(test)]
mod tests {
    use cutler::defaults::{get_flag_and_value, get_flag_for_value, normalize_desired};
    use toml::Value;

    #[test]
    fn test_get_flag_and_value() {
        // Test boolean values
        let (flag, value) = get_flag_and_value(&Value::Boolean(true)).unwrap();
        assert_eq!(flag, "-bool");
        assert_eq!(value, "true");

        let (flag, value) = get_flag_and_value(&Value::Boolean(false)).unwrap();
        assert_eq!(flag, "-bool");
        assert_eq!(value, "false");

        // Test integer values
        let (flag, value) = get_flag_and_value(&Value::Integer(42)).unwrap();
        assert_eq!(flag, "-int");
        assert_eq!(value, "42");

        // Test float values
        let (flag, value) = get_flag_and_value(&Value::Float(3.14)).unwrap();
        assert_eq!(flag, "-float");
        assert_eq!(value, "3.14");

        // Test string values
        let (flag, value) = get_flag_and_value(&Value::String("test".to_string())).unwrap();
        assert_eq!(flag, "-string");
        assert_eq!(value, "test");
    }

    #[test]
    fn test_get_flag_for_value() {
        // Test boolean-like strings
        let (flag, value) = get_flag_for_value("true").unwrap();
        assert_eq!(flag, "-bool");
        assert_eq!(value, "true");

        let (flag, value) = get_flag_for_value("0").unwrap();
        assert_eq!(flag, "-bool");
        assert_eq!(value, "0");

        // Test integer-like strings
        let (flag, value) = get_flag_for_value("42").unwrap();
        assert_eq!(flag, "-int");
        assert_eq!(value, "42");

        // Test float-like strings
        let (flag, value) = get_flag_for_value("3.14").unwrap();
        assert_eq!(flag, "-float");
        assert_eq!(value, "3.14");

        // Test regular strings
        let (flag, value) = get_flag_for_value("test string").unwrap();
        assert_eq!(flag, "-string");
        assert_eq!(value, "test string");
    }

    #[test]
    fn test_normalize_desired() {
        // Test boolean normalization
        assert_eq!(normalize_desired(&Value::Boolean(true)), "1");
        assert_eq!(normalize_desired(&Value::Boolean(false)), "0");

        // Test string normalization
        assert_eq!(
            normalize_desired(&Value::String("test".to_string())),
            "test"
        );

        // Test number normalization
        assert_eq!(normalize_desired(&Value::Integer(42)), "42");
        assert_eq!(normalize_desired(&Value::Float(3.14)), "3.14");
    }
}
