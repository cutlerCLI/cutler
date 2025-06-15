use crate::util::globals::should_be_quiet;

/// ANSI color codes
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const PINK: &str = "\x1b[35m";
pub const ORANGE: &str = "\x1b[38;5;208m";
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

/// Logging level based on what action cutler is performing.
#[derive(PartialEq)]
pub enum LogLevel {
    Success,
    Error,
    Warning,
    Info,
    CommandOutput,
    Dry,
    Fruitful, // 🍎
}

/// Central logger.
/// It is important that most, if not all, prints in cutler go through this function.
pub fn print_log(level: LogLevel, msg: &str) {
    if should_be_quiet() && level != LogLevel::Error && level != LogLevel::Warning {
        return;
    }

    let (tag, color) = match level {
        LogLevel::Success => ("SUCCESS", GREEN),
        LogLevel::Error => ("ERROR", RED),
        LogLevel::Warning => ("WARNING", YELLOW),
        LogLevel::Info => ("INFO", BOLD),
        LogLevel::CommandOutput => ("CMD OUT", PINK),
        LogLevel::Dry => ("DRY-RUN", ORANGE),
        LogLevel::Fruitful => ("🍎", ""), // Apple emoji, no color
    };

    let line = if level == LogLevel::Fruitful {
        format!("{} {}", tag, msg)
    } else {
        format!("{}[{}]{} {}", color, tag, RESET, msg)
    };

    if level == LogLevel::Error || level == LogLevel::Warning {
        eprintln!("{}", line);
    } else {
        println!("{}", line);
    }
}
