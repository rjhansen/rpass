//! rpass is a modern, memory-safe, zeroizing, cryptographically-strong
//! alternative to Ted Ts’o’s `pwgen` password generator. It is almost
//! a drop-in replacement.

mod password;
mod cmdline;
mod hardening;
mod printer;
mod signals;
mod terminal;

use cmdline::get_count;
use printer::make_printer;
use password::PasswordGenerator;
use std::process::ExitCode;

/// Does all the work. Everything holding sensitive data lives in this
/// frame, so it's dropped (and zeroized) before `main` reports an error
/// or acts on a signal.
fn run() -> Result<(), String> {
    hardening::disable_core_dumps()?;
    signals::install()?;
    let mut pw = PasswordGenerator::new()?;
    let mut printer = make_printer();

    for _ in 0..get_count() {
        if signals::pending().is_some() || printer(pw.generate())?.is_break() {
            break;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let result = run();
    if let Some(signo) = signals::pending() {
        // Everything sensitive is gone now, so die from the signal just
        // as we would have without a handler. Shells rely on seeing that.
        signals::die_from(signo);
        return ExitCode::from(u8::try_from(128 + signo).unwrap_or(u8::MAX));
    }
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}
