#[cfg(test)]
mod tests {
    use cutler::external::execute_external_commands;
    use toml::Value;
    use toml::value::Table;

    // This is a more complex test that would require mocking
    // the process execution. In a real implementation, we might
    // use a crate like mockall to mock the Command execution.
    // For simplicity, I'll provide a basic structure:

    #[test]
    fn test_execute_external_commands_dry_run() {
        // Create a test TOML structure with external commands
        let mut variables = Table::new();
        variables.insert(
            "hostname".to_string(),
            Value::String("test-host".to_string()),
        );

        let mut cmd1 = Table::new();
        cmd1.insert("cmd".to_string(), Value::String("echo".to_string()));
        cmd1.insert(
            "args".to_string(),
            Value::Array(vec![
                Value::String("Hello".to_string()),
                Value::String("$hostname".to_string()),
            ]),
        );

        let mut external = Table::new();
        external.insert("variables".to_string(), Value::Table(variables));
        external.insert(
            "command".to_string(),
            Value::Array(vec![Value::Table(cmd1)]),
        );

        let mut root = Table::new();
        root.insert("external".to_string(), Value::Table(external));

        let config = Value::Table(root);

        // Test in dry-run mode (should not execute anything)
        let result = execute_external_commands(&config, true, true);
        assert!(result.is_ok());
    }
}
