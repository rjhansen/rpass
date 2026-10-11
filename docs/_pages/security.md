---
title: "Security"
permalink: /security/
excerpt: "Security considerations."
---

`rpass` is released without any guarantees. However, I've made modest
efforts to harden it in light of the evolving security landscape.

* **Zeroizing.** All sensitive memory is zeroized as soon as it's no
  longer needed.
* **Signals.** Upon receiving `SIGTERM` or `Ctrl-C`, sensitive memory
  gets wiped.
* **Aborts.** In the extremely unusual case of Rust calling an abort,
  sensitive memory gets wiped.
* **Core dumps.** Core dumps are disabled for non-root users.
* **Debuggers.** You'll need to be root to attach a debugger.
