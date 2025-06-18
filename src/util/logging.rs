use crate::util::globals::{is_verbose, should_be_quiet};

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
    Error,
    Warning,
    Info,
    Prompt, // only for io::confirm_action()
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

    if (level == LogLevel::Info || level == LogLevel::CommandOutput) && !is_verbose() {
        return;
    }

    let (tag, color) = match level {
        LogLevel::Error => ("ERROR", RED),
        LogLevel::Warning => ("WARNING", YELLOW),
        LogLevel::Info => ("INFO", BOLD),
        LogLevel::CommandOutput => ("CMD OUT", PINK),
        LogLevel::Prompt => ("PROMPT", PINK),
        LogLevel::Dry => ("DRY-RUN", ORANGE),
        LogLevel::Fruitful => ("🍎", ""),
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
