use clap::Parser;
use cutler::cli::Args;
use cutler::commands::dispatch;

fn main() {
    let args = Args::parse();

    if let Err(err) = dispatch(
        &args.command,
        args.verbose,
        args.dry_run,
        args.no_restart_services,
    ) {
        eprintln!("❌ error: {}", err);
        std::process::exit(1);
    }
}
