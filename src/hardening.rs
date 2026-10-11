//! Keeps the operating system from writing our memory out to disk if we
//! crash. Zeroizing only helps with memory we get the chance to wipe; a
//! core dump taken mid-run would capture the glyph pool and the CSPRNG
//! key regardless.
//!
//! This is only implemented for UNIX. Elsewhere it's a no-op.

/// Turns off core dumps for this process. Call this before any sensitive
/// data exists.
///
/// # Errors
///
/// Returns a message for the user if the operating system refuses. That
/// should never happen, so we'd rather stop than carry on unprotected.
#[cfg_attr(
    not(target_family = "unix"),
    allow(clippy::unnecessary_wraps, reason = "it can only fail on UNIX")
)]
pub fn disable_core_dumps() -> Result<(), String> {
    #[cfg(target_family = "unix")]
    {
        use rustix::process::{Resource, Rlimit, setrlimit};
        // Zero for both the soft and the hard limit, so nothing later in
        // the process can raise it again.
        let no_core = Rlimit {
            current: Some(0),
            maximum: Some(0),
        };
        setrlimit(Resource::Core, no_core)
            .map_err(|e| format!("could not disable core dumps: {e}"))?;
    }

    // The limit above isn't enough on Linux: per core(5), it “is not
    // enforced for core dumps that are piped to a program,” which is how
    // systemd-coredump and apport get them. Marking ourselves non-dumpable
    // covers that, and also keeps other processes of the same user from
    // attaching to us with ptrace.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        use rustix::process::{DumpableBehavior, set_dumpable_behavior};
        set_dumpable_behavior(DumpableBehavior::NotDumpable)
            .map_err(|e| format!("could not mark the process as non-dumpable: {e}"))?;
    }

    Ok(())
}
