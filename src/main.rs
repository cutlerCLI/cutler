use clap::{Parser, Subcommand};
use cutler::{
    apply_defaults, delete_config, print_log, restart_system_services, status_defaults,
    unapply_defaults, LogLevel, RED, RESET,
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
    /// Display current status comparing the config vs current defaults.
    Status,
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Apply => apply_defaults(cli.verbose),
        Commands::Unapply => unapply_defaults(cli.verbose),
        Commands::Delete => delete_config(cli.verbose),
        Commands::Status => status_defaults(cli.verbose),
    };

    match result {
        Ok(_) => {
            // Restart system services for commands that modify defaults.
            match &cli.command {
                Commands::Apply | Commands::Unapply | Commands::Delete => {
                    if let Err(e) = restart_system_services(cli.verbose) {
                        eprintln!("{}[ERROR]{} Failed to restart services: {}", RED, RESET, e);
                    }
                }
                Commands::Status => {}
            }
        }
        Err(e) => {
            print_log(LogLevel::Error, &format!("{}", e), cli.verbose);
            std::process::exit(1);
        }
    }
}
