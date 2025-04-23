/// Color constants.
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const PINK: &str = "\x1b[35m";
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

/// Log levels for printing messages.
#[derive(PartialEq)]
pub enum LogLevel {
    Success,
    Error,
    Warning,
    Info,
    CommandOutput,
}

/// Central logging function.
pub fn print_log(level: LogLevel, message: &str) {
    let (prefix, color) = match level {
        LogLevel::Success => ("SUCCESS", GREEN),
        LogLevel::Error => ("ERROR", RED),
        LogLevel::Warning => ("WARNING", YELLOW),
        LogLevel::Info => ("INFO", BOLD),
        LogLevel::CommandOutput => ("CMD OUT", PINK),
    };

    let formatted = format!("{}[{}]{} {}", color, prefix, RESET, message);

    if level == LogLevel::Error || level == LogLevel::Warning {
        eprintln!("{}", formatted);
    } else {
        println!("{}", formatted);
    }
}
