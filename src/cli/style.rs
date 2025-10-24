// SPDX-License-Identifier: Apache-2.0

use anstyle::{AnsiColor, Color::Ansi, Style};

/// Style attributes for cutler CLI.
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Ansi(AnsiColor::Black)))
                .bg_color(Some(Ansi(AnsiColor::White))),
        )
        .header(
            Style::new()
                .bold()
                .bg_color(Some(Ansi(AnsiColor::White)))
                .fg_color(Some(Ansi(AnsiColor::Black))),
        )
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
