---
title: "Installation"
permalink: /installation/
excerpt: "Building and installing rpass from source."
---

## Supported systems

* Apple macOS (tested on an M4 MacBook Pro running macOS 26.5.1 (Tahoe))
* Linux (tested on an x86_64 machine running Fedora 43)
* Windows (tested on an x86_64 machine running Windows 11)

## Intended audience

System administrators. This tool probably isn't useful to regular users.

## Installing from source

To install from source you'll need the [Rust development tools](https://rust-lang.org/learn/get-started/).
Once those are installed, `rpass` can be installed through `cargo` with the
following:

```shell
$ cargo install --git https://github.com/rjhansen/rpass
```

Next: read the [usage guide](/usage/) for the full flag reference and a
quickstart, or the [security analysis](/security/) if you want to understand
how `rpass` protects the passwords it generates.
