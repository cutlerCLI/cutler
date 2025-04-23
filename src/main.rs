use clap::Parser;
use cutler::{
    cli::{Cli, Commands, ConfigCommand},
    commands::{
        apply_defaults, check_for_updates, config_delete, config_show,
        execute_only_external_commands, init_config, reset_defaults, status_defaults,
        unapply_defaults,
    },
    completions, helpers,
    logging::{LogLevel, print_log},
};

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Apply => apply_defaults(cli.verbose, cli.dry_run),
        Commands::Cmd => execute_only_external_commands(cli.verbose, cli.dry_run),
        Commands::Init { force } => init_config(cli.verbose, *force),
        Commands::Unapply => unapply_defaults(cli.verbose, cli.dry_run),
        Commands::Reset { force } => reset_defaults(cli.verbose, cli.dry_run, *force),
        Commands::Status => status_defaults(cli.verbose),
        Commands::Config { command } => match command {
            ConfigCommand::Show => config_show(cli.verbose, cli.dry_run),
            ConfigCommand::Delete => config_delete(cli.verbose, cli.dry_run),
        },
        Commands::Completion { shell } => completions::generate_completion(*shell),
        Commands::CheckUpdate => check_for_updates(cli.verbose),
    };

    match result {
        Ok(_) => {
            // For commands that modify defaults, restart system services.
            match &cli.command {
                Commands::Apply
                | Commands::Unapply
                | Commands::Reset { .. }
                | Commands::Config {
                    command: ConfigCommand::Delete,
                } => {
                    if !cli.no_restart_services {
                        if let Err(e) = helpers::restart_system_services(cli.verbose, cli.dry_run) {
                            eprintln!("🍎 Manual restart might be required: {}", e);
                        }
                    }
                }
                _ => {}
            }
        }
        Err(e) => {
            print_log(LogLevel::Error, &format!("{}", e));
            std::process::exit(1);
        }
    }
}
