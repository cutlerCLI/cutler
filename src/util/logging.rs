// SPDX-License-Identifier: GPL-3.0-or-later

//! Logging module for cutler.
//!
//! Use the log_*! macros for pretty-printing text inside cutler.

use crate::cli::atomic::{should_be_quiet, should_be_verbose};

// ANSI color codes.
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const PINK: &str = "\x1b[35m";
pub const ORANGE: &str = "\x1b[38;5;208m";
pub const CYAN: &str = "\x1b[36m";
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

#[doc(hidden)]
#[derive(PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Prompt, // only for io::confirm()
    Exec,
    Dry,
    Fruitful, // 🍎
}

#[doc(hidden)]
pub fn _print_log(level: LogLevel, msg: &str) {
    if (should_be_quiet() && level != LogLevel::Error && level != LogLevel::Warning)
        || (level == LogLevel::Info && !should_be_verbose())
    {
        return;
    }

    let (tag, color) = match level {
        LogLevel::Error => ("ERR  ", RED),
        LogLevel::Warning => ("WARN ", ORANGE),
        LogLevel::Info => ("INFO ", CYAN),
        LogLevel::Exec => ("EXEC ->", RED),
        LogLevel::Prompt => ("PRMT ", PINK),
        LogLevel::Dry => ("DRY  ", YELLOW),
        LogLevel::Fruitful => ("🍎", ""),
    };

    let line = if level == LogLevel::Fruitful {
        format!("{tag} {msg}")
    } else {
        format!("{color}{tag}{RESET} {msg}")
    };

    if level == LogLevel::Error || level == LogLevel::Warning {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}

/// Logs with LogLevel::Info.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        $crate::util::logging::_print_log($crate::util::logging::LogLevel::Info, &msg);
    }};
}

/// Logs with LogLevel::Error.
#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        $crate::util::logging::_print_log($crate::util::logging::LogLevel::Error, &msg);
    }};
}

/// Logs with LogLevel::Warning.
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        $crate::util::logging::_print_log($crate::util::logging::LogLevel::Warning, &msg);
    }};
}

/// Logs with LogLevel::Fruitful.
#[macro_export]
macro_rules! log_cute {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        $crate::util::logging::_print_log($crate::util::logging::LogLevel::Fruitful, &msg);
    }};
}

/// Logs with LogLevel::Dry.
#[macro_export]
macro_rules! log_dry {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        $crate::util::logging::_print_log($crate::util::logging::LogLevel::Dry, &msg);
    }};
}

/// Logs with LogLevel::Exec.
#[macro_export]
macro_rules! log_exec {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        $crate::util::logging::_print_log($crate::util::logging::LogLevel::Exec, &msg);
    }};
}

/// Logs with LogLevel::Prompt.
#[macro_export]
macro_rules! log_prompt {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        $crate::util::logging::_print_log($crate::util::logging::LogLevel::Prompt, &msg);
    }};
}
