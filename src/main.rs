// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::exit;

use clap::Parser;
use cutler::autosync::try_auto_sync;

use cutler::cli::Args;
use cutler::cli::atomic::{
    set_accept_all, set_dry_run, set_no_restart_services, set_quiet, set_verbose,
};
use cutler::commands::Runnable;
use cutler::config::Config;
use cutler::config::get_config_path;
use cutler::util::sudo::{run_with_noroot, run_with_root};
use cutler::{log_err, log_info};

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

    // decide configuration path for the entire lifetime of the program
    let mut config = if let Ok(path) = get_config_path().await {
        Config::new(path)
    } else {
        log_err!("Path could not be decided for the configuration file.");
        exit(1);
    };

    // remote config auto-sync logic
    if args.no_sync {
        log_info!("Skipping remote config autosync.");
    } else {
        try_auto_sync(&args.command, &mut config).await;
    }

    // command invocation (for real this time)
    let runnable: &dyn Runnable = args.command.as_runnable();

    // sudo protection
    if let Err(e) = if runnable.needs_sudo() {
        run_with_root().await
    } else {
        run_with_noroot()
    } {
        log_err!("{e}");
        exit(1);
    }

    let result = runnable.run(&mut config).await;

    if let Err(err) = result {
        log_err!("{err}");
        exit(1);
    }
}
