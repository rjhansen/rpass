use crate::cmdline;
use cmdline::Args;
use terminal_size::{terminal_size, Height, Width};

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
    let tw = get_terminal_width();
    let len = args.length.unwrap_or(8);
    if tw <= len {
        return 1;
    }
    let tw = f32::from(tw);
    let l = f32::from(len + 1);
    ((tw / l).ceil() as u16).saturating_sub(2).max(1)
}
