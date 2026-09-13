---
layout: splash
title: "rpass"
excerpt: "Because random passwords shouldn't be left to chance."
header:
  overlay_color: "#1a1a2e"
  overlay_filter: "0.4"
  actions:
    - label: "Get Started"
      url: /installation/
    - label: "View on GitHub"
      url: https://github.com/rjhansen/rpass
feature_row:
  - title: "Cryptographically secure"
    excerpt: "Every password is drawn from a ChaCha20-based CSPRNG, with six bits of entropy guaranteed per glyph."
  - title: "Aggressive zeroization"
    excerpt: "Sensitive buffers — random bytes, intermediate strings, and printed passwords — are zeroized as soon as they're no longer needed."
  - title: "Drop-in for pwgen"
    excerpt: "Familiar flags for anyone who's used pwgen, without inheriting its dated cryptography or low-entropy defaults."
---

{% include feature_row %}

## What is rpass?

`rpass` is a command-line password generator built for system administrators
who need cryptographically strong passwords without fuss. It's a spiritual
successor to Ted Ts'o's `pwgen`, keeping the command-line ergonomics
sysadmins already know while replacing the underlying cryptography and
entropy guarantees with something suited to today's threat environment.

- [**Installation**]({{ '/installation/' | relative_url }}) — build from source with `cargo`.
- [**Usage**]({{ '/usage/' | relative_url }}) — the full flag reference, quickstart examples, and answers to "why does it work this way?"

## Quickstart

```shell
$ cargo install --git https://github.com/rjhansen/rpass
$ rpass -1 16 8
golZPZzCYZEzyUOn
5c81wvlT/58lGDpu
rw1sqWlOLJQ0dYQy
qSJBM9EkSZAEFjtI
A8Tb/YZhz5eQ3ErP
YIJAiwgR25utOq69
oOXCufI9DcdjyXsf
0Zf3cZsJ/4vFjc0l
```

`rpass` by itself generates twenty lines of columnar eight-character
passwords; `rpass -1` generates just one. See the [usage guide]({{ '/usage/' | relative_url }}) for
every flag.
