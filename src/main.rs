use clap::Parser;
use cutler::autosync::try_auto_sync;
use cutler::cli::args::{BrewSubcmd, ConfigSubcmd};
use cutler::cli::completion::generate_completion;
use cutler::cli::{Args, Command};
use cutler::commands::Runnable;
use cutler::util::globals::{
    set_accept_interactive, set_dry_run, set_no_restart_services, set_quiet, set_verbose,
};
use cutler::util::logging::{LogLevel, print_log};
use cutler::util::sudo::{run_with_noroot, run_with_root};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    // set some of them atomically
    // (described why in util/globals.rs)
    set_accept_interactive(args.accept_interactive);
    set_quiet(args.quiet);
    set_verbose(args.verbose);
    set_dry_run(args.dry_run);
    set_no_restart_services(args.no_restart_services);

    // remote config auto-sync (if enabled)
    try_auto_sync(&args.command).await;

    // sudo protection
    let result = match &args.command {
        Command::SelfUpdate(_) => run_with_root(),
        _ => run_with_noroot(),
    };

    if let Err(err) = result {
        print_log(LogLevel::Error, &format!("Invoke failure: {err}"));
        std::process::exit(1);
    }

    // command invocation (for real this time)
    let result = match &args.command {
        Command::Apply(cmd) => cmd.run().await,
        Command::Exec(cmd) => cmd.run().await,
        Command::Init(cmd) => cmd.run().await,
        Command::Unapply(cmd) => cmd.run().await,
        Command::Reset(cmd) => cmd.run().await,
        Command::Status(cmd) => cmd.run().await,
        Command::Config { command } => match command {
            ConfigSubcmd::Show(cmd) => cmd.run().await,
            ConfigSubcmd::Delete(cmd) => cmd.run().await,
            ConfigSubcmd::Lock(cmd) => cmd.run().await,
            ConfigSubcmd::Unlock(cmd) => cmd.run().await,
            ConfigSubcmd::Sync(cmd) => cmd.run().await,
        },
        Command::Brew { command } => match command {
            BrewSubcmd::Backup(cmd) => cmd.run().await,
            BrewSubcmd::Install(cmd) => cmd.run().await,
        },
        Command::CheckUpdate(cmd) => cmd.run().await,
        Command::SelfUpdate(cmd) => cmd.run().await,
        Command::Completion { shell } => generate_completion(shell.to_owned()).await,
    };

    if let Err(err) = result {
        print_log(LogLevel::Error, &err.to_string());
        std::process::exit(1);
    }
}
