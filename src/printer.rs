//! Abstracts away OS differences in how to most safely use standard
//! output. On UNIX we use the `rustix` library to safely call the
//! `libc` `write()` syscall directly in order to guarantee we avoid
//! any buffering that may be in place with Rust's normal `stdout()`.
//!
//! `rustix` doesn't exist for Windows, so there we just give best effort.

use std::process::exit;
use crate::cmdline::{get_count, parse_command_line};
use crate::terminal::get_words_per_line;
use zeroize::Zeroize;

#[cfg(target_family = "unix")]
fn zeroize_writeln(mut word: String) {
    let stream = rustix::stdio::stdout();
    if rustix::io::write(stream, word.as_bytes()).is_ok() { () } else {
        eprintln!("error: failed to write to stdout!");
        exit(1);
    }
    if rustix::io::write(stream, b"\n").is_ok() { () } else {
        eprintln!("error: failed to write to stdout!");
        exit(1);
    }
    word.zeroize();
}

#[cfg(target_family = "unix")]
fn zeroize_writespc(mut word: String) {
    let stream = rustix::stdio::stdout();
    if rustix::io::write(stream, word.as_bytes()).is_ok() { () } else {
        eprintln!("error: failed to write to stdout!");
        exit(1);
    };
    if rustix::io::write(stream, b" ").is_ok() { () } else {
        eprintln!("error: failed to write to stdout!");
        exit(1);
    };
    word.zeroize();
}

#[cfg(target_family = "windows")]
fn zeroize_writeln(mut word: String) {
    println!("{word}");
    word.zeroize();
}

#[cfg(target_family = "windows")]
fn zeroize_writespc(mut word: String) {
    print!("{word} ");
    word.zeroize();
}

/// This closure encapsulates all the logic involved in displaying a
/// password to the screen. Note the function it returns assumes
/// ownership of the password so it can zeroize it post-display.
pub fn make_printer() -> impl FnMut(String) {
    let args = parse_command_line();
    let count = get_count();
    let wpl = get_words_per_line(args.length.unwrap_or(8));
    let mut index = 0;
    let mut remaining_in_line = wpl;

    move |password: String| {
        let is_last = index >= count.saturating_sub(1);

        if !args.multi_column || is_last || remaining_in_line == 0 {
            zeroize_writeln(password);
            remaining_in_line = wpl;
        } else {
            zeroize_writespc(password);
            remaining_in_line -= 1;
        }
        index += 1;
    }
}

