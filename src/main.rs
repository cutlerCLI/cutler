// SPDX-License-Identifier: Apache-2.0

use std::process::exit;

use clap::Parser;
use cutler::autosync::try_auto_sync;
use cutler::cli::args::{BrewSubcmd, ConfigSubcmd};

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

    // sudo protection
    let result = match &args.command {
        Command::SelfUpdate(_)
        | Command::Config {
            command: ConfigSubcmd::Lock(_) | ConfigSubcmd::Unlock(_),
        } => run_with_root().await,
        _ => run_with_noroot(),
    };

    if let Err(err) = result {
        print_log(LogLevel::Error, &format!("Invoke failure: {err}"));
        exit(1);
    }

    // command invocation (for real this time)
    let result = match &args.command {
        Command::Apply(cmd) => cmd.run().await,
        Command::Cookbook(cmd) => cmd.run().await,
        Command::Exec(cmd) => cmd.run().await,
        Command::Fetch(cmd) => cmd.run().await,
        Command::Init(cmd) => cmd.run().await,
        Command::Unapply(cmd) => cmd.run().await,
        Command::Reset(cmd) => cmd.run().await,
        Command::Status(cmd) => cmd.run().await,
        Command::Config { command } => match command {
            ConfigSubcmd::Show(cmd) => cmd.run().await,
            ConfigSubcmd::Delete(cmd) => cmd.run().await,
            ConfigSubcmd::Lock(cmd) => cmd.run().await,
            ConfigSubcmd::Unlock(cmd) => cmd.run().await,
        },
        Command::Brew { command } => match command {
            BrewSubcmd::Backup(cmd) => cmd.run().await,
            BrewSubcmd::Install(cmd) => cmd.run().await,
        },
        Command::CheckUpdate(cmd) => cmd.run().await,
        Command::SelfUpdate(cmd) => cmd.run().await,
        Command::Completion(cmd) => cmd.run().await,
    };

    if let Err(err) = result {
        print_log(LogLevel::Error, &err.to_string());
        exit(1);
    }
}
