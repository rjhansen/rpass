//! Abstracts away OS differences in how to most safely use standard
//! output. On UNIX we use the `rustix` library to safely call the
//! `libc` `write()` syscall directly in order to guarantee we avoid
//! any buffering that may be in place with Rust's normal `stdout()`.
//!
//! `rustix` doesn't exist for Windows, so there we just give best effort.

use crate::cmdline::{get_count, parse_command_line};
use crate::terminal::get_words_per_line;
use zeroize::Zeroize;

#[cfg(target_family = "unix")]
fn zeroize_writeln(mut foo: String) {
    let _ = rustix::io::write(rustix::stdio::stdout(), foo.as_bytes());
    let _ = rustix::io::write(rustix::stdio::stdout(), "\n".as_bytes());
    foo.zeroize();
}

#[cfg(target_family = "unix")]
fn zeroize_writespc(mut foo: String) {
    let _ = rustix::io::write(rustix::stdio::stdout(), foo.as_bytes());
    let _ = rustix::io::write(rustix::stdio::stdout(), " ".as_bytes());
    foo.zeroize();
}

#[cfg(target_family = "windows")]
fn zeroize_writeln(mut foo: String) {
    let _ = std::io::write(std::io::stdout(), foo.as_bytes());
    let _ = std::io::write(std::io::stdout(), "\n".as_bytes());
    foo.zeroize();
}

#[cfg(target_family = "windows")]
fn zeroize_writespc(mut foo: String) {
    let _ = std::io::write(std::io::stdout(), foo.as_bytes());
    let _ = std::io::write(std::io::stdout(), " ".as_bytes());
    foo.zeroize();
}

/// This closure encapsultes all the logic involved in displaying a
/// password to the screen. Note the function it returns assumes
/// ownership of the password so it can zeroize it post-display.
pub fn make_printer() -> impl FnMut(String) {
    let args = parse_command_line();
    let count = get_count();
    let wpl = get_words_per_line();
    let mut index = 0;
    let mut remaining_in_line = wpl;

    move |password: String| {
        let is_last = index >= count.saturating_sub(1);

        if !args.multi_column || is_last {
            zeroize_writeln(password);
        } else {
            remaining_in_line -= 1;
            if remaining_in_line == 0 {
                zeroize_writeln(password);
                remaining_in_line = wpl;
            } else {
                zeroize_writespc(password);
            }
        }
        index += 1;
    }
}

