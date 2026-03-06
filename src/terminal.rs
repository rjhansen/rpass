use crate::cmdline;
use cmdline::Args;
use terminal_size::{Height, Width, terminal_size};

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

#[must_use]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub fn get_words_per_line(args: &Args) -> u16 {
    if get_terminal_width() < args.length.unwrap_or(8) {
        1
    } else {
        let tw = f32::from(get_terminal_width());
        let l = f32::from(args.length.unwrap_or(8) + 1);
        ((tw / l).ceil() as u16) - 2
    }
}
