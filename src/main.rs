//! rpass is a modern, memory-safe, zeroizing, cryptographically-strong
//! alternative to Ted Ts’o’s `pwgen` password generator. It is almost
//! a drop-in replacement.

mod password;
mod cmdline;
mod printer;
mod terminal;

use cmdline::get_count;
use printer::make_printer;
use password::PasswordGenerator;

fn main() {
    let mut pw = PasswordGenerator::new();
    let mut printer = make_printer();

    for _ in 0..get_count() {
        printer(pw.generate());
    }
}
