//! Provides basic tools for probing terminal parameters.

use terminal_size::{terminal_size, Height, Width};

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
pub fn get_words_per_line(len: u16) -> u16 {
    let tw = get_terminal_width();
    if tw <= len {
        return 1;
    }
    let tw = f32::from(tw);
    let l = f32::from(len + 1);
    ((tw / l).floor() as u16).max(1)
}
