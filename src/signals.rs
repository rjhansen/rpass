//! Lets Ctrl-C and friends end the program by the normal return path, so
//! that everything holding sensitive data gets dropped (and zeroized) on
//! the way out instead of being abandoned in memory.
//!
//! The handler only records which signal arrived. The main loop and the
//! write loop poll for it, unwind normally, and then `main` re-raises the
//! signal so our parent still sees us die from it.
//!
//! This is only implemented for UNIX. Elsewhere these are no-ops and a
//! signal ends the program the way it always did.

#[cfg(target_family = "unix")]
mod unix {
    use std::sync::atomic::{AtomicI32, Ordering};

    /// The signal we've been asked to die from, or 0 if there isn't one.
    static PENDING: AtomicI32 = AtomicI32::new(0);

    /// The signal handler. Storing to an atomic is async-signal-safe;
    /// almost nothing else is, so this must stay this simple.
    extern "C" fn note_signal(signo: libc::c_int) {
        PENDING.store(signo, Ordering::Relaxed);
    }

    pub fn install() -> Result<(), String> {
        for signo in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
            // SAFETY: `sigaction` is a plain C struct for which all-zeroes
            // is a valid value, and every pointer passed below is either
            // null or refers to a live local.
            let installed = unsafe {
                let mut current: libc::sigaction = std::mem::zeroed();
                if libc::sigaction(signo, std::ptr::null(), &raw mut current) != 0 {
                    false
                } else if current.sa_sigaction == libc::SIG_IGN {
                    // We were started with this signal ignored (`nohup`, for
                    // instance). Respect that instead of un-ignoring it.
                    true
                } else {
                    let mut action: libc::sigaction = std::mem::zeroed();
                    action.sa_sigaction = note_signal as extern "C" fn(libc::c_int) as usize;
                    // Deliberately no SA_RESTART: a write that's blocked on
                    // a full pipe has to come back with EINTR, or we'd never
                    // get the chance to notice the signal.
                    action.sa_flags = 0;
                    libc::sigemptyset(&raw mut action.sa_mask);
                    libc::sigaction(signo, &raw const action, std::ptr::null_mut()) == 0
                }
            };
            if !installed {
                return Err(format!("could not install a handler for signal {signo}"));
            }
        }
        Ok(())
    }

    pub fn pending() -> Option<i32> {
        match PENDING.load(Ordering::Relaxed) {
            0 => None,
            signo => Some(signo),
        }
    }

    pub fn die_from(signo: i32) {
        // SAFETY: restoring a signal's default action and raising it are
        // both always sound; the worst case is that they fail and return.
        unsafe {
            libc::signal(signo, libc::SIG_DFL);
            libc::raise(signo);
        }
    }
}

/// Arranges for SIGINT, SIGTERM and SIGHUP to be recorded instead of
/// killing us outright. Call this before any sensitive data exists.
///
/// # Errors
///
/// Returns a message for the user if a handler can't be installed.
#[cfg_attr(
    not(target_family = "unix"),
    allow(clippy::unnecessary_wraps, reason = "it can only fail on UNIX")
)]
pub fn install() -> Result<(), String> {
    #[cfg(target_family = "unix")]
    unix::install()?;
    Ok(())
}

/// Returns the signal we've been asked to die from, if there is one.
#[must_use]
pub fn pending() -> Option<i32> {
    #[cfg(target_family = "unix")]
    return unix::pending();
    #[cfg(not(target_family = "unix"))]
    None
}

/// Dies from `signo` the way we would have had we never caught it. Only
/// call this once everything sensitive has been dropped. It returns only
/// if the signal somehow failed to kill us.
pub fn die_from(signo: i32) {
    #[cfg(target_family = "unix")]
    unix::die_from(signo);
    #[cfg(not(target_family = "unix"))]
    let _ = signo;
}
