use clap::Parser;
use cutler::cli::Args;
use cutler::commands::dispatch;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    // Set quiet flag globally
    cutler::util::logging::set_quiet(args.quiet);

    if let Err(err) = dispatch(
        &args.command,
        args.verbose,
        args.dry_run,
        args.no_restart_services,
        args.accept_all,
        args.quiet,
    )
    .await
    {
        eprintln!("❌ error: {}", err);
        std::process::exit(1);
    }
}
