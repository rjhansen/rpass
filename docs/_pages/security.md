---
title: "Security"
permalink: /security/
excerpt: "Security considerations."
---

`rpass` is released without any guarantees. However, I've made modest
efforts to harden it in light of the evolving security landscape.

* **Zeroizing.** With only a few exceptions, sensitive memory is zeroized
  as soon as it's no longer needed. Exceptions include transient copies on
  the stack, in registers, and anything living in kernel memory. If you're
  displaying passwords they also continue to live in your graphics hardware.
  Also, it may get swapped out: if that concerns you, encrypt your swap.
* **Signals.** (UNIX-only) Upon receiving `SIGTERM`, `SIGHUP`, or `Ctrl-C`,
  sensitive memory gets wiped.
* **Panics.** In the extremely unusual case of Rust calling a panic,
  sensitive memory gets wiped.
* **Core dumps.** (UNIX-only) Core dumps are disabled.
* **Debuggers.** (Linux-only) You'll need to be root to attach a debugger.
