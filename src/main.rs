//! rpass is a modern, memory-safe, zeroizing, cryptographically-strong
//! alternative to Ted Ts’o’s `pwgen` password generator. It is almost
//! a drop-in replacement.

pub mod character_generator;
pub mod cmdline;
pub mod printer;
pub mod terminal;

use cmdline::get_count;
use printer::make_printer;
use crate::character_generator::PasswordGenerator;

fn main() {
    let mut pw = PasswordGenerator::new();
    // The printer closure encapsulates details such as terminal width,
    // how many to print on a line, and so forth.
    let mut printer = make_printer();

    for _ in 0..get_count() {
        printer(pw.generate());
    }
}
