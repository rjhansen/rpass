//! Runs the real `rpass` binary and checks what a user or a shell
//! pipeline would see: its output, its exit status, and how it behaves
//! when the pipe it's writing to closes or a signal arrives.
//!
//! Every child is started with no terminal attached, so `rpass` falls
//! back to its 80-column default and the layout is predictable.

#![allow(
    clippy::tests_outside_test_module,
    reason = "an integration test crate is nothing but tests"
)]

use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const RPASS: &str = env!("CARGO_BIN_EXE_rpass");

/// Runs `rpass` to completion and collects everything it printed.
fn rpass(args: &[&str]) -> Output {
    Command::new(RPASS)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("rpass should start")
}

/// Starts `rpass` with pipes on stdout and stderr and leaves it running.
fn spawn(args: &[&str]) -> Child {
    Command::new(RPASS)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rpass should start")
}

/// Waits for `child` to exit, giving up after `limit`.
fn wait_up_to(child: &mut Child, limit: Duration) -> Option<ExitStatus> {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait().expect("child should be waitable") {
            return Some(status);
        }
        if Instant::now() >= deadline {
            return None;
        }
        sleep(Duration::from_millis(10));
    }
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

/// Checks that `rpass` refused `args` with exit status 1, said nothing on
/// stdout, and explained itself on stderr with a message containing
/// `expected`.
fn assert_refused(args: &[&str], expected: &str) {
    let output = rpass(args);
    let stderr = stderr_of(&output);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{args:?} should exit with status 1; stderr was: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "{args:?} was refused but still printed: {}",
        stdout_of(&output)
    );
    assert!(
        stderr.starts_with("error: ") && stderr.contains(expected),
        "{args:?} was refused with the wrong message: {stderr}"
    );
}

// ---- output --------------------------------------------------------------

#[test]
fn one_column_prints_one_password_per_line() {
    let output = rpass(&["-1", "20", "5"]);
    assert!(output.status.success(), "rpass -1 20 5 should succeed");
    assert!(
        output.stderr.is_empty(),
        "a successful run should print nothing on stderr, but printed: {}",
        stderr_of(&output)
    );
    let stdout = stdout_of(&output);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 5, "expected five passwords, got: {stdout:?}");
    for line in lines {
        assert_eq!(line.len(), 20, "{line:?} isn't twenty glyphs long");
    }
    assert!(stdout.ends_with('\n'), "output should end with a newline");
}

#[test]
fn one_column_alone_prints_a_single_password() {
    let stdout = stdout_of(&rpass(&["-1"]));
    assert_eq!(
        stdout.lines().count(),
        1,
        "expected one line, got: {stdout:?}"
    );
    assert_eq!(
        stdout.trim_end().len(),
        8,
        "expected the default length of 8"
    );
}

#[test]
fn multicolumn_fills_eighty_columns() {
    // Nine 8-glyph passwords and eight spaces make exactly 80 columns.
    let stdout = stdout_of(&rpass(&["-C", "8", "36"]));
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "36 passwords should fill four lines: {stdout:?}"
    );
    for line in lines {
        let words: Vec<&str> = line.split(' ').collect();
        assert_eq!(words.len(), 9, "{line:?} should hold nine passwords");
        assert_eq!(line.len(), 80, "{line:?} should be exactly 80 columns");
        for word in words {
            assert_eq!(word.len(), 8, "{word:?} isn't eight glyphs long");
        }
    }
}

#[test]
fn multicolumn_ends_a_short_last_line_with_a_newline() {
    let stdout = stdout_of(&rpass(&["8", "11"]));
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "11 passwords should take two lines: {stdout:?}"
    );
    assert_eq!(
        lines[1].split(' ').count(),
        2,
        "the second line should hold the two left over: {stdout:?}"
    );
    assert!(
        stdout.ends_with('\n') && !stdout.ends_with(" \n"),
        "the last line should end with a newline and no trailing space: {stdout:?}"
    );
}

#[test]
fn default_run_prints_twenty_full_lines() {
    let stdout = stdout_of(&rpass(&[]));
    assert_eq!(stdout.lines().count(), 20, "expected twenty lines");
    assert_eq!(
        stdout.split_whitespace().count(),
        180,
        "expected twenty lines of nine passwords"
    );
}

#[test]
fn largest_request_is_delivered_in_full() {
    let stdout = stdout_of(&rpass(&["-1", "43", "10000"]));
    assert_eq!(stdout.lines().count(), 10_000, "expected 10,000 passwords");
    for line in stdout.lines() {
        assert_eq!(line.len(), 43, "{line:?} isn't 43 glyphs long");
    }
}

#[test]
fn slow_reader_loses_nothing() {
    // Reading a few bytes at a time keeps the pipe full, so rpass's
    // writes are constantly blocking and resuming.
    let mut child = spawn(&["-1", "43", "2000"]);
    let mut stdout = child.stdout.take().expect("stdout should be piped");
    let mut received = Vec::new();
    let mut chunk = [0u8; 7];
    loop {
        let count = stdout.read(&mut chunk).expect("pipe should be readable");
        if count == 0 {
            break;
        }
        received.extend_from_slice(&chunk[..count]);
    }
    let status = child.wait().expect("child should be waitable");
    assert!(status.success(), "rpass should succeed with a slow reader");
    let text = String::from_utf8(received).expect("stdout should be UTF-8");
    assert_eq!(text.lines().count(), 2000, "expected 2,000 passwords");
    for line in text.lines() {
        assert_eq!(line.len(), 43, "{line:?} isn't 43 glyphs long");
    }
}

// ---- informational flags ---------------------------------------------------

#[test]
fn version_bugs_and_copyright_succeed() {
    let version = rpass(&["--version"]);
    assert!(version.status.success(), "--version should succeed");
    assert_eq!(
        stdout_of(&version).trim_end(),
        format!("rpass {}", env!("CARGO_PKG_VERSION")),
        "--version should print the package version"
    );

    let bugs = rpass(&["--bugs"]);
    assert!(bugs.status.success(), "--bugs should succeed");
    assert!(
        stdout_of(&bugs).contains("https://github.com/rjhansen/rpass"),
        "--bugs should say where to report them"
    );

    let copyright = rpass(&["--copyright"]);
    assert!(copyright.status.success(), "--copyright should succeed");
    assert!(
        stdout_of(&copyright).contains("Apache 2.0"),
        "--copyright should name the license"
    );
}

// ---- refusals ----------------------------------------------------------------

#[test]
fn conflicting_flags_are_refused() {
    assert_refused(&["-c", "-A"], "conflicting flags");
    assert_refused(&["-n", "-0"], "conflicting flags");
}

#[test]
fn out_of_range_length_is_refused() {
    assert_refused(&["5"], "length must be in the range [6, 43]");
    assert_refused(&["44"], "length must be in the range [6, 43]");
}

#[test]
fn out_of_range_count_is_refused() {
    assert_refused(&["8", "0"], "count must be in range [1, 10000]");
    assert_refused(&["8", "10001"], "count must be in range [1, 10000]");
}

#[test]
fn too_little_entropy_is_refused() {
    assert_refused(&["-A", "6"], "entropy guarantee of 36 bits");
    assert_refused(
        &["-A", "-0", "-v", "-B", "8"],
        "entropy guarantee of 36 bits",
    );
}

#[test]
fn impossible_requirements_are_refused() {
    assert_refused(
        &["-n", "-r", "0123456789", "12"],
        "numbers are required, but all were excluded",
    );
}

#[test]
fn unknown_flags_are_a_usage_error() {
    let output = rpass(&["-H"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "pwgen's -H isn't supported and should be a usage error"
    );
    assert!(
        output.stdout.is_empty(),
        "a usage error should print no passwords"
    );
}

// ---- pipelines ---------------------------------------------------------------

#[test]
fn reader_going_away_is_a_quiet_success() {
    // This is `rpass -1 43 10000 | head -1`.
    let mut child = spawn(&["-1", "43", "10000"]);
    let mut reader = BufReader::new(child.stdout.take().expect("stdout should be piped"));
    let mut first = String::new();
    reader
        .read_line(&mut first)
        .expect("pipe should be readable");
    assert_eq!(first.trim_end().len(), 43, "expected one 43-glyph password");
    drop(reader);

    let status = wait_up_to(&mut child, Duration::from_secs(5))
        .expect("rpass should exit once its reader has gone");
    assert!(
        status.success(),
        "a closed pipe should not be an error: {status}"
    );

    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr should be piped")
        .read_to_string(&mut stderr)
        .expect("stderr should be readable");
    assert!(
        stderr.is_empty(),
        "a closed pipe should be quiet, but got: {stderr}"
    );
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::fs::File;
    use std::os::unix::process::ExitStatusExt;

    const SIGHUP: i32 = 1;
    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;

    fn send(signal: &str, child: &Child) {
        let delivered = Command::new("kill")
            .args([format!("-{signal}"), child.id().to_string()])
            .status()
            .expect("kill should run");
        assert!(delivered.success(), "kill -{signal} should succeed");
    }

    /// Starts a run far too big for a pipe's buffer and never reads from
    /// it, so `rpass` ends up blocked inside `write()`. Keep the returned
    /// pipe alive for as long as the child should stay blocked.
    fn spawn_blocked() -> (Child, std::process::ChildStdout) {
        let mut child = spawn(&["-1", "43", "10000"]);
        let stdout = child.stdout.take().expect("stdout should be piped");
        sleep(Duration::from_millis(300));
        assert!(
            child
                .try_wait()
                .expect("child should be waitable")
                .is_none(),
            "rpass should be blocked on the full pipe, not finished"
        );
        (child, stdout)
    }

    /// Checks that a blocked `rpass` dies promptly from the signal it was
    /// sent, which is what tells a shell to stop the surrounding loop.
    fn assert_dies_from(name: &str, number: i32) {
        let (mut child, _stdout) = spawn_blocked();
        send(name, &child);
        let status = wait_up_to(&mut child, Duration::from_secs(5))
            .unwrap_or_else(|| panic!("rpass ignored SIG{name} while blocked on a pipe"));
        assert_eq!(
            status.signal(),
            Some(number),
            "rpass should die from SIG{name}, but ended with: {status}"
        );
    }

    #[test]
    fn sigint_ends_a_blocked_run() {
        assert_dies_from("INT", SIGINT);
    }

    #[test]
    fn sigterm_ends_a_blocked_run() {
        assert_dies_from("TERM", SIGTERM);
    }

    #[test]
    fn sighup_ends_a_blocked_run() {
        assert_dies_from("HUP", SIGHUP);
    }

    #[test]
    fn ignored_signal_stays_ignored() {
        // `trap ''` makes the shell ignore SIGINT, and an ignored signal
        // stays ignored across exec. This is what `nohup` does for SIGHUP.
        let mut child = Command::new("sh")
            .args(["-c", "trap '' INT; exec \"$0\" -1 43 10000", RPASS])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("sh should start");
        let stdout = child.stdout.take().expect("stdout should be piped");
        sleep(Duration::from_millis(300));

        send("INT", &child);
        assert!(
            wait_up_to(&mut child, Duration::from_millis(500)).is_none(),
            "rpass should carry on when started with SIGINT ignored"
        );

        // Closing the pipe lets it finish the usual quiet way.
        drop(stdout);
        let status = wait_up_to(&mut child, Duration::from_secs(5))
            .expect("rpass should exit once its reader has gone");
        assert!(status.success(), "expected a quiet success, got: {status}");
    }

    #[test]
    fn unwritable_stdout_is_an_error() {
        // A read-only descriptor makes every write fail outright.
        let read_only = File::open("/dev/null").expect("/dev/null should open");
        let output = Command::new(RPASS)
            .args(["-1", "8", "1"])
            .stdin(Stdio::null())
            .stdout(read_only)
            .stderr(Stdio::piped())
            .output()
            .expect("rpass should start");
        assert_eq!(
            output.status.code(),
            Some(1),
            "a failed write should exit with status 1"
        );
        assert!(
            stderr_of(&output).contains("failed to write to stdout"),
            "a failed write should be reported, but stderr was: {}",
            stderr_of(&output)
        );
    }
}
