// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use cutler::{
        cli::atomic::set_dry_run,
        config::core::{Command, Config},
        exec::core::{ExecMode, run_all, run_one},
    };
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_run_all_dry_run() {
        set_dry_run(true);

        // Build a [vars] table
        let mut vars = HashMap::new();
        vars.insert("hostname".into(), "test-host".into());

        // Build a [command.foo] table
        let mut command_map = HashMap::new();
        command_map.insert(
            "foo".into(),
            Command {
                run: "echo Hello $hostname".into(),
                ensure_first: None,
                required: None,
                flag: None,
                sudo: None,
            },
        );

        // Top-level config
        let config = Config {
            lock: None,
            set: None,
            vars: Some(vars),
            command: Some(command_map),
            brew: None,
            mas: None,
            remote: None,
            path: Default::default(),
        };

        assert!(run_all(config, ExecMode::Regular).await.is_ok());
    }

    #[tokio::test]
    async fn test_run_one_dry_run() {
        set_dry_run(true);

        // Build a [vars] table
        let mut vars = HashMap::new();
        vars.insert("USER".into(), "me".into());

        // Build a [command.whoami] table
        let mut command_map = HashMap::new();
        command_map.insert(
            "whoami".into(),
            Command {
                run: "echo $USER".into(),
                ensure_first: None,
                required: None,
                flag: None,
                sudo: Some(true),
            },
        );

        // Top-level config
        let config = Config {
            lock: None,
            set: None,
            vars: Some(vars),
            command: Some(command_map),
            brew: None,
            mas: None,
            remote: None,
            path: Default::default(),
        };

        // Dry‑run single command
        assert!(run_one(config, "whoami").await.is_ok());
    }
}
