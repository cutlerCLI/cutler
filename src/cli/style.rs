// SPDX-License-Identifier: MIT OR Apache-2.0

use anstyle::{AnsiColor, Color::Ansi, Effects, Style};

/// Style attributes for cutler CLI.
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(Style::new().effects(Effects::CURLY_UNDERLINE).bold())
        .header(Style::new().effects(Effects::CURLY_UNDERLINE).bold())
        .literal(Style::new().bold())
        .invalid(Style::new().bold().fg_color(Some(Ansi(AnsiColor::Red))))
        .error(Style::new().bold().fg_color(Some(Ansi(AnsiColor::Red))))
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Ansi(AnsiColor::White))))
}
