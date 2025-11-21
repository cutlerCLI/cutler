// SPDX-License-Identifier: MIT OR Apache-2.0

/* The Heart *sparkles*
 *
 * Some personal take:
 *
 * I've tried incorporating a very minimal main.rs since most developers often try to fill their projects' main.rs up
 * with unnecessary boilerplate. cutler has been my primary way of learning and practicing Rust through a real-world
 * project which I actually use on a daily basis in my workflow, and honestly it has made me a better programmer to
 * say the least.
 *
 * However, I do not like people who flood crates.io with unnecessary iterations of their own garbage, so let's spread
 * good code and let's spread "meaningful" code.
 *
 * I don't know how well I've even managed to write cutler (as any amount of code is a liability anyway), but hopefully
 * if you're looking at this and you've cloned the project onto your computer, you'll have a look around, and maybe
 * produce a lot of good code, which could potentially be beneficial to both you and the community.
 *
 * I relicensed this code from MIT to GPL only from the sheer inspiration of writing "good" code (taken from other
 * projects such as git), so I suppose let's keep that going ^w^ happy coding!
 */

use std::process::exit;

use clap::Parser;
use cutler::autosync::try_auto_sync;

use cutler::cli::atomic::{
    set_accept_all, set_dry_run, set_no_restart_services, set_quiet, set_verbose,
};
use cutler::cli::{Args, Command};
use cutler::commands::Runnable;
use cutler::config::core::Config;
use cutler::config::path::get_config_path;
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
    let mut config = if let Ok(path) = get_config_path().await { Config::new(path) } else {
        log_err!("Path could not be decided for the configuration file.");
        exit(1);
    };

    // remote config auto-sync logic
    if args.no_sync {
        log_info!("Skipping remote config autosync.");
    } else {
        try_auto_sync(&args.command, &mut config).await;
    }

    // sudo protection
    let result = match &args.command {
        Command::SelfUpdate(_) | Command::Lock(_) | Command::Unlock(_) => run_with_root().await,
        _ => run_with_noroot(),
    };

    if let Err(err) = result {
        log_err!("{err}");
        exit(1);
    }

    // command invocation (for real this time)
    let runnable: &dyn Runnable = args.command.as_runnable();
    let result = runnable.run(&mut config).await;

    if let Err(err) = result {
        log_err!("{err}");
        exit(1);
    }
}
