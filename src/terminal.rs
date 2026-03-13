//! Provides basic tools for probing terminal parameters.

use terminal_size::{terminal_size, Height, Width};
use crate::cmdline::parse_command_line;

/// Returns the terminal width, if sane, or a reasonable alternative,
/// if not.
#[must_use]
pub fn get_terminal_width() -> u16 {
    match terminal_size() {
        Some((Width(w), Height(_))) => {
            if w <= 7 {
                7
            } else if w < 160 {
                w
            } else {
                160
            }
        }
        None => 80,
    }
}

/// Answers the age-old question, “how many passwords of the user’s
/// specified length can fit on a line of this terminal?”
#[must_use]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub fn get_words_per_line() -> u16 {
    let args = parse_command_line();
    let tw = get_terminal_width();
    let len = args.length.unwrap_or(8);
    if tw <= len {
        return 1;
    }
    let tw = f32::from(tw + 1);
    let l = f32::from(len + 1);
    ((tw / l).floor() as u16).max(1)
}
