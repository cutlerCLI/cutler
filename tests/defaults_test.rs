#[cfg(test)]
mod tests {
    use cutler::defaults::{from_flag, normalize, to_flag};
    use toml::Value;

    #[test]
    fn test_to_flag() {
        // Boolean
        let (flag, value) = to_flag(&Value::Boolean(true)).unwrap();
        assert_eq!(flag, "-bool");
        assert_eq!(value, "true");
        let (flag, value) = to_flag(&Value::Boolean(false)).unwrap();
        assert_eq!(flag, "-bool");
        assert_eq!(value, "false");

        // Integer
        let (flag, value) = to_flag(&Value::Integer(42)).unwrap();
        assert_eq!(flag, "-int");
        assert_eq!(value, "42");

        // Float
        let (flag, value) = to_flag(&Value::Float(3.14)).unwrap();
        assert_eq!(flag, "-float");
        assert_eq!(value, "3.14");

        // String
        let (flag, value) = to_flag(&Value::String("test".into())).unwrap();
        assert_eq!(flag, "-string");
        assert_eq!(value, "test");
    }

    #[test]
    fn test_from_flag() {
        // Boolean‐like
        let (flag, val) = from_flag("true").unwrap();
        assert_eq!(flag, "-bool");
        assert_eq!(val, "true");
        let (flag, val) = from_flag("0").unwrap();
        assert_eq!(flag, "-bool");
        assert_eq!(val, "0");

        // Integer‐like
        let (flag, val) = from_flag("42").unwrap();
        assert_eq!(flag, "-int");
        assert_eq!(val, "42");

        // Float‐like
        let (flag, val) = from_flag("3.14").unwrap();
        assert_eq!(flag, "-float");
        assert_eq!(val, "3.14");

        // Fallback string
        let (flag, val) = from_flag("hello world").unwrap();
        assert_eq!(flag, "-string");
        assert_eq!(val, "hello world");
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize(&Value::Boolean(true)), "1");
        assert_eq!(normalize(&Value::Boolean(false)), "0");
        assert_eq!(normalize(&Value::String("foo".into())), "foo");
        assert_eq!(normalize(&Value::Integer(5)), "5");
        assert_eq!(normalize(&Value::Float(2.5)), "2.5");
    }
}
