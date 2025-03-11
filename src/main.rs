// Imports.
use clap::{Parser, Subcommand};
use cutler::{
    apply_defaults, delete_config, print_log, restart_system_services, unapply_defaults, LogLevel,
    RED, RESET,
};

/// Declarative macOS settings management at your fingertips, with speed.
#[derive(Parser)]
#[command(name = "cutler", version, about)]
struct Cli {
    /// Increase output verbosity
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply defaults from the config file.
    Apply,
    /// Unapply (delete) defaults from the config file.
    Unapply,
    /// Delete the configuration file.
    Delete,
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Apply => apply_defaults(cli.verbose),
        Commands::Unapply => unapply_defaults(cli.verbose),
        Commands::Delete => delete_config(cli.verbose),
    };

    match result {
        Ok(_) => {
            print_log(LogLevel::Success, "Done!", true);
            if let Err(e) = restart_system_services(cli.verbose) {
                eprintln!("{}[ERROR]{} Failed to restart services: {}", RED, RESET, e);
            }
        }
        Err(e) => {
            print_log(LogLevel::Error, &format!("{}", e), true);
            std::process::exit(1);
        }
    }
}
