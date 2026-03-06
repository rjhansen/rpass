use crate::cmdline;
use terminal_size::{terminal_size, Height, Width};

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

pub fn get_words_per_line(args: &cmdline::Args) -> u16 {
    ((get_terminal_width() as f32 / (args.length.unwrap_or_else(|| 8) + 1) as f32).ceil() as u16)
        - 2
}
