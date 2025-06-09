/// ANSI color codes
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const PINK: &str = "\x1b[35m";
pub const ORANGE: &str = "\x1b[38;5;208m";
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

use std::sync::atomic::{AtomicBool, Ordering};

#[derive(PartialEq)]
pub enum LogLevel {
    Success,
    Error,
    Warning,
    Info,
    CommandOutput,
    Dry,
}

// Global quiet flag
static QUIET: AtomicBool = AtomicBool::new(false);

pub fn set_quiet(value: bool) {
    QUIET.store(value, Ordering::SeqCst);
}

pub fn get_quiet() -> bool {
    QUIET.load(Ordering::SeqCst)
}

/// Central logger.
/// It is important that most, if not all, prints in cutler go through this function.
pub fn print_log(level: LogLevel, msg: &str) {
    if get_quiet() && level != LogLevel::Error && level != LogLevel::Warning {
        return;
    }

    let (tag, color) = match level {
        LogLevel::Success => ("SUCCESS", GREEN),
        LogLevel::Error => ("ERROR", RED),
        LogLevel::Warning => ("WARNING", YELLOW),
        LogLevel::Info => ("INFO", BOLD),
        LogLevel::CommandOutput => ("CMD OUT", PINK),
        LogLevel::Dry => ("DRY-RUN", ORANGE),
    };

    let line = format!("{}[{}]{} {}", color, tag, RESET, msg);

    if level == LogLevel::Error || level == LogLevel::Warning {
        eprintln!("{}", line);
    } else {
        println!("{}", line);
    }
}
