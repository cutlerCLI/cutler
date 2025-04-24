#[cfg(test)]
mod tests {
    use cutler::external::run_all;
    use toml::{Value, value::Table};

    #[test]
    fn test_run_all_dry_run() {
        let mut vars = Table::new();
        vars.insert("hostname".into(), Value::String("test-host".into()));

        let mut cmd = Table::new();
        cmd.insert("cmd".into(), Value::String("echo".into()));
        cmd.insert(
            "args".into(),
            Value::Array(vec![
                Value::String("Hello".into()),
                Value::String("$hostname".into()),
            ]),
        );

        let mut ext = Table::new();
        ext.insert("variables".into(), Value::Table(vars));
        ext.insert("command".into(), Value::Array(vec![Value::Table(cmd)]));

        let config = Value::Table({
            let mut m = Table::new();
            m.insert("external".into(), Value::Table(ext));
            m
        });

        // Should succeed in dry‐run
        assert!(run_all(&config, true, true).is_ok());
    }
}
