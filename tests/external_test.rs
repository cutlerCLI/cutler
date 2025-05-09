#[cfg(test)]
mod tests {
    use cutler::external::runner::{run_all, run_one};
    use toml::{Value, value::Table};

    #[tokio::test]
    async fn test_run_all_dry_run() {
        // Build a [vars] table
        let mut vars = Table::new();
        vars.insert("hostname".into(), Value::String("test-host".into()));

        // Build a [commands.foo] table
        let mut cmd = Table::new();
        cmd.insert("run".into(), Value::String("echo Hello $hostname".into()));
        // sudo is optional; default is false
        let mut commands = Table::new();
        commands.insert("foo".into(), Value::Table(cmd));

        // Top‐level config = { vars = {...}, commands = { foo = { … } } }
        let mut root = Table::new();
        root.insert("vars".into(), Value::Table(vars));
        root.insert("commands".into(), Value::Table(commands));
        let config = Value::Table(root);

        // Dry‑run should always succeed
        assert!(
            run_all(&config, /*verbose=*/ true, /*dry_run=*/ true)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_run_one_dry_run() {
        // Very similar setup
        let mut vars = Table::new();
        vars.insert("USER".into(), Value::String("me".into()));

        let mut cmd = Table::new();
        cmd.insert("run".into(), Value::String("echo $USER".into()));
        // mark it sudo=true to exercise that branch
        cmd.insert("sudo".into(), Value::Boolean(true));

        let mut commands = Table::new();
        commands.insert("whoami".into(), Value::Table(cmd));

        let mut root = Table::new();
        root.insert("vars".into(), Value::Table(vars));
        root.insert("commands".into(), Value::Table(commands));
        let config = Value::Table(root);

        // Dry‑run single command
        assert!(
            run_one(
                &config, "whoami", /*verbose=*/ true, /*dry_run=*/ true
            )
            .await
            .is_ok()
        );
    }
}
