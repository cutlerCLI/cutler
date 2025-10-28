// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::process::exit;

use clap::Parser;
use cutler::autosync::try_auto_sync;
use cutler::cli::args::BrewSubcmd;

use cutler::cli::atomic::{
    set_accept_all, set_dry_run, set_no_restart_services, set_quiet, set_verbose,
};
use cutler::cli::{Args, Command};
use cutler::commands::Runnable;
use cutler::util::logging::{LogLevel, print_log};
use cutler::util::sudo::{run_with_noroot, run_with_root};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    // set some of them atomically
    // (described why in util/globals.rs)
    set_accept_all(args.accept_all);
    set_quiet(args.quiet);
    set_verbose(args.verbose);
    set_dry_run(args.dry_run);
    set_no_restart_services(args.no_restart_services);

    // remote config auto-sync logic
    if !args.no_sync {
        try_auto_sync(&args.command).await;
    } else {
        print_log(LogLevel::Info, "Skipping remote config auto-sync.");
    }

    if env::var("CUTLER_NO_HINTS").is_err() {
        print_log(
            LogLevel::Warning,
            "Run `cutler brew backup` if you are using Homebrew backups in cutler as new release contains breaking changes.",
        );
        print_log(
            LogLevel::Warning,
            "Suppress this warning by exporting `CUTLER_NO_HINTS=1` in your shell.",
        );
    }

    // sudo protection
    let result = match &args.command {
        Command::SelfUpdate(_) | Command::Lock(_) | Command::Unlock(_) => run_with_root().await,
        _ => run_with_noroot(),
    };

    if let Err(err) = result {
        print_log(LogLevel::Error, &err.to_string());
        exit(1);
    }

    // command invocation (for real this time)
    let runnable: &dyn Runnable = match &args.command {
        Command::Apply(cmd) => cmd,
        Command::Config(cmd) => cmd,
        Command::Cookbook(cmd) => cmd,
        Command::Exec(cmd) => cmd,
        Command::Fetch(cmd) => cmd,
        Command::Init(cmd) => cmd,
        Command::Unapply(cmd) => cmd,
        Command::Reset(cmd) => cmd,
        Command::Status(cmd) => cmd,
        Command::Lock(cmd) => cmd,
        Command::Unlock(cmd) => cmd,
        Command::Brew { command } => match command {
            BrewSubcmd::Backup(cmd) => cmd,
            BrewSubcmd::Install(cmd) => cmd,
        },
        Command::CheckUpdate(cmd) => cmd,
        Command::SelfUpdate(cmd) => cmd,
        Command::Completion(cmd) => cmd,
    };
    let result = runnable.run().await;

    if let Err(err) = result {
        print_log(LogLevel::Error, &err.to_string());
        exit(1);
    }
}
