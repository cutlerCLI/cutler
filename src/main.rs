use clap::Parser;
use cutler::cli::args::{BrewSubcmd, ConfigSubcmd};
use cutler::cli::completion::generate_completion;
use cutler::cli::{Args, Command};
use cutler::commands::{GlobalArgs, Runnable};
use cutler::util::globals::{set_accept_interactive, set_quiet};
use cutler::util::logging::{LogLevel, print_log};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    // collect global arguments
    let globals = GlobalArgs {
        verbose: args.verbose,
        dry_run: args.dry_run,
        quiet: args.quiet,
        no_restart_services: args.no_restart_services,
        accept_interactive: args.accept_interactive,
    };

    // set some of them atomically
    // (described why in util/globals.rs)
    set_accept_interactive(args.accept_interactive);
    set_quiet(args.quiet);

    let result = match &args.command {
        Command::Apply(cmd) => cmd.run(&globals).await,
        Command::Exec(cmd) => cmd.run(&globals).await,
        Command::Init(cmd) => cmd.run(&globals).await,
        Command::Unapply(cmd) => cmd.run(&globals).await,
        Command::Reset(cmd) => cmd.run(&globals).await,
        Command::Status(cmd) => cmd.run(&globals).await,
        Command::Config { command } => match command {
            ConfigSubcmd::Show(cmd) => cmd.run(&globals).await,
            ConfigSubcmd::Delete(cmd) => cmd.run(&globals).await,
        },
        Command::Brew { command } => match command {
            BrewSubcmd::Backup(cmd) => cmd.run(&globals).await,
            BrewSubcmd::Install(cmd) => cmd.run(&globals).await,
        },
        Command::CheckUpdate(cmd) => cmd.run(&globals).await,
        Command::SelfUpdate(cmd) => cmd.run(&globals).await,
        Command::Completion { shell } => generate_completion(shell.to_owned()).await,
    };

    if let Err(err) = result {
        print_log(LogLevel::Error, &err.to_string());
        std::process::exit(1);
    }
}
