//! Abstracts away OS differences in how to most safely use standard
//! output. On UNIX we use the `rustix` library to safely call the
//! `libc` `write()` syscall directly in order to guarantee we avoid
//! any buffering that may be in place with Rust's normal `stdout()`.
//!
//! `rustix` doesn't exist for Windows, so there we just give best effort.

use crate::cmdline::{get_count, parse_command_line};
#[cfg(target_family = "unix")]
use crate::signals;
use crate::terminal::get_words_per_line;
use std::ops::ControlFlow;
use zeroize::Zeroizing;

/// Tells the caller whether there's any point in printing more. `Break`
/// means either that whoever was reading our output has gone away
/// (`rpass | head -1`), which is a normal way for a pipeline to end, or
/// that we've been signalled to stop. Neither is an error.
type WriteResult = Result<ControlFlow<()>, String>;

#[cfg(target_family = "unix")]
fn write_stdout(word: &str, ch: &str) -> WriteResult {
    use rustix::io::Errno;
    let stream = rustix::stdio::stdout();
    for mut bytes in [word.as_bytes(), ch.as_bytes()] {
        // A single write() may take only part of what it's given, or be
        // interrupted by a signal before taking any, so keep going until
        // it's all gone.
        while !bytes.is_empty() {
            if signals::pending().is_some() {
                return Ok(ControlFlow::Break(()));
            }
            match rustix::io::write(stream, bytes) {
                Ok(0) => return Err("failed to write to stdout!".into()),
                Ok(written) => bytes = &bytes[written..],
                Err(Errno::PIPE) => return Ok(ControlFlow::Break(())),
                // Either it's a signal we handle, which the check at the
                // top of the loop will catch, or it's one we don't care
                // about and we should just try again.
                Err(Errno::INTR) => {}
                Err(_) => return Err("failed to write to stdout!".into()),
            }
        }
    }
    Ok(ControlFlow::Continue(()))
}

#[cfg(target_family = "windows")]
fn write_stdout(word: &str, ch: &str) -> WriteResult {
    use std::io::{ErrorKind, Write};
    let mut stream = std::io::stdout().lock();
    match stream
        .write_all(word.as_bytes())
        .and_then(|()| stream.write_all(ch.as_bytes()))
        .and_then(|()| stream.flush())
    {
        Ok(()) => Ok(ControlFlow::Continue(())),
        Err(e) if e.kind() == ErrorKind::BrokenPipe => Ok(ControlFlow::Break(())),
        Err(_) => Err("failed to write to stdout!".into()),
    }
}

/// This closure encapsulates all the logic involved in displaying a
/// password to the screen. Note the function it returns assumes
/// ownership of the password, so the password is zeroized as soon as
/// it's been displayed (or has failed to be). It
/// reports a failed write as an error message for the user, and a
/// closed pipe or a pending signal as `Break`, meaning “stop quietly.”
pub fn make_printer() -> impl FnMut(Zeroizing<String>) -> WriteResult {
    let args = parse_command_line();
    let count = get_count();
    let wpl = get_words_per_line(args.length.unwrap_or(8));
    let mut index = 0;
    let mut remaining_in_line = wpl;

    move |password: Zeroizing<String>| {
        let is_last = index >= count.saturating_sub(1);

        let result = if !args.multi_column || is_last || remaining_in_line == 0 {
            remaining_in_line = wpl;
            write_stdout(&password, "\n")
        } else {
            remaining_in_line -= 1;
            write_stdout(&password, " ")
        };
        index += 1;
        result
    }
}
