# rpass

## _… because random passwords shouldn’t be left to chance._

## Supported systems

* Apple macOS (tested on an M4 MacBook Pro running macOS 26.3.1 (Tahoe))
* Linux (tested on an x86_64 machine running Fedora 43)
* Windows (tested on an x86_64 machine running Windows 11)

## Intended audience

System administrators. This tool probably isn’t useful to regular users.

## Installation

To install from source you’ll need the [Rust development tools](https://rust-lang.org/learn/get-started/).
Once those are installed, `rpass` can be installed through `cargo` with the
following:

```shell
$ cargo install --git https://github.com/rjhansen/rpass
```

## What problem does it solve?

Many system administrators need a command-line utility to produce random passwords.
The central problem here is that much of the popular guidance is
[genuinely bad](https://www.howtogeek.com/30184/10-ways-to-generate-a-random-password-from-the-command-line/).
(No, you really shouldn’t pipe `date` to `md5sum` and use that.) The current king
of the quality password generators is Ted Ts’o’s `pwgen`, but even that has a few
shortcomings. To wit,

* It’s unmaintained
* It uses cryptography from the last century
* It produces low-entropy passwords

That last one requires some explanation in order to be fair to Ted. He never
intended `pwgen` to make guarantees about its output’s resistance to brute force
attacks. He was aiming to produce good-enough passwords people could easily
remember. By producing passwords that could be pronounced (roughly, kinda, sorta),
he was able to produce outputs that were good enough for the threats of the day
and were also memorable enough they wouldn’t need to be written on a Post-It note
and attached to the user’s monitor.

We don’t live in that era any more. Passwords need to be resistant to brute force
attacks, dictionary attacks, rainbow tables, and more. Users are encouraged to use
password managers to help them keep track of all the different passwords they need
to do their daily business. Finally, advanced persistent threats exist in many 
sysadmins’ environments, and password generators need to be written to be aggressive
in how they zeroize memory before returning it to the system.

With respect and a grateful nod of the head to Ted Ts’o for `pwgen`, an overhaul is 
needed.

## Quickstart

`rpass` usage should be familiar to anyone who’s used `pwgen`.

`rpass -1 16 8` displays sixteen-character passwords, eight of them, in a single column:

```shell
❯ rpass -1 16 8
golZPZzCYZEzyUOn
5c81wvlT/58lGDpu
rw1sqWlOLJQ0dYQy
qSJBM9EkSZAEFjtI
A8Tb/YZhz5eQ3ErP
YIJAiwgR25utOq69
oOXCufI9DcdjyXsf
0Zf3cZsJ/4vFjc0l
```
`rpass -C 8 40` displays eight-character passwords, forty of them, in multiple columns.
(`rpass` is terminal-aware, so you might get more or fewer columns based on the width
of your terminal.)

```shell
❯ rpass -C 8 36
xVumsL1b VCpdj+Z/ lfTQna5Q dTNMCcu9 YJ5l57tU 1EWhV/Bn EkxNa/qg Chwh+bH9 BB3uGC6B
hZ8d0cI3 8UtaMLMt gztBPcRa 6po3ePbU 4sYCicyy yuyNGEIY INpqTvc0 d+jvYOWp 7IzrdLZx
SBD/0/Ol wcgGzW4U /nUKaS7C 6tmUNTdw S3bn5Bc3 OJg8TGTM YAcVs1mv tuhvpgKQ +y/yaqot
UQeK6Ppi Aei5P4ec tsCaTs4E ahdsQbMS EA8LCPC4 dsg9Jv6F ljBofVtt M/EtC8qT ReBf4U0a
```

By default, `rpass` by itself will generate twenty lines of columnar eight-character
passwords. `rpass -1` by itself will generate just one password.

## Usage

`rpass` is, as far as is reasonable, a drop-in replacement for `pwgen`. These
command-line arguments should look familiar:

```
rpass 1.2.1
Generates high-entropy passwords.

Usage: rpass [OPTIONS] [LENGTH] [COUNT]

Arguments:
  [LENGTH]
  [COUNT]

Options:
  -c, --capitalize             Ensure one or more capital letters
  -A, --no-capitalize          Ensure no capital letters
  -n, --numerals               Ensure one or more numbers
  -0, --no-numerals            Ensure no numbers
  -y, --symbols                Ensure at least one special character
  -r, --remove-chars <REMOVE>  Omit these characters [default: ""]
  -s, --secure                 Generate completely random passwords
  -B, --ambiguous              Don't include ambiguous characters
  -v, --no-vowels              Don't use any vowels
  -©, --copyright              Show copyright notice
  -1, --one-column             Display results in one column
  -C, --multicolumn            Display results in multiple columns
  -b, --bugs                   Where to file bug reports
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
  ```

A few of these flags deserve special comment.

* The short form of `--copyright` uses the Unicode copyright symbol, ©.
  If your terminal doesn’t support Unicode, use the full `--copyright` flag.
* Flags that ensure the output will conform to content policies will reduce the
  overall password strength by making it more predictable. Account for this and
  choose a longer password if needed.
* `-s` does nothing: `rpass` works securely by default and has no less-safe mode.
* When using `-C`, `rpass` will be reasonably accommodating of terminal sizes but
  not unreasonably so. If your terminal width is set to, say, 2500, calibrate your
  expectations accordingly.
* Technically, `-C` is a no-op because that’s the default behavior anyway. However,
  `pwgen` works this way, so we do too in the interests of being a drop-in
  replacement.
* `pwgen`’s `-H` flag is not supported and **will not be** supported (not by me,
  at least). It was borderline-foolish even when Ted first wrote `pwgen`, and in
  today’s threat environment it’s genuinely foolish.
* Using `-r` will invalidate our `six bits per glyph!` entropy guarantee. In fact,
  unwise use of `-r` can drop the entropy per glyph to dangerously low levels. To
  see what is meant by that, consider what would happen if one were to pass
  `--remove-chars=abcdefghijklmnopqrstuvwxyz`: how many bits of entropy per glyph
  would the output possess then? Removing twenty-six glyphs from the base64 output
  would leave us with 38 different symbols, for about 5.25 bits of entropy per
  glyph — significantly below our six bits per glyph guarantee!
* `LENGTH` must be in the range [6, 43] inclusive.
* `COUNT` must be in the range [1, 10000] inclusive.

## Why…

### … is there a cap on how many passwords can be generated?

It’s overwhelmingly likely that requests for trillions of passwords are the result
of a cat walking on the sysadmin’s keyboard rather than an actual request from the
sysadmin. As an anti-cat measure, we assume any request for more than 10,000
passwords is feline in origin.

### … is there a cap on password length?

Because 256-bit passwords will be secure until computers are made of something
other than matter, occupy something other than space, run in something other than
time, according to something other than the laws of thermodynamics.

According to our best understanding of the universe, non-adiabatic computing (the 
kind we know how to do efficiently: all our computers, even Google's cutting-edge
quantum ones, are non-adiabatic) flat-out requires a certain amount of heat to be
liberated each time you erase a bit of information. That number is really small:
to be precise, it’s _kT ln(2)_ (the Boltzmann constant multiplied by the temperature of
the environment multiplied by the natural logarithm of two) per bit, but it’s not zero.

| Constant |                                          Value                                           |
|:--------:|:----------------------------------------------------------------------------------------:|
|  **k**   |  [1.380649 ⋅ 10⁻²³ joules per kelvin](https://en.wikipedia.org/wiki/Boltzmann_constant)  |
|   **T**  | [2.725 K](https://bigthink.com/starts-with-a-bang/life-begin-universe-room-temperature/) |
| **ln(2)**|                                    0.6931471805599453                                    |

Each time you want to try a new key you have to erase about half the bits you were
using from your last run. For a 256-bit key, that’s going to be on average about
2⁸ erasures. You have to do this for 2²⁵⁵ different keys (half the 256-bit keyspace).
In total, you’re paying that very small thermodynamic tax 2²⁶³ times — or about 10⁷⁷.

(Could you cool the computer to a nanokelvin? Sure, but then you’re paying more tax to
run the heat pump. There is no escaping the universe’s taxman.)

1.380649 joules per kelvin ⋅ 2.725 kelvins ⋅ 10⁻²³ ⋅ 10⁷⁷ ≈ 2.61 ⋅ 10⁵⁴ joules

The largest energy unit I’m familiar with is the foe, which represents the average
energy liberated in a supernova explosion. One foe is 10⁴⁴ joules.

2.61 ⋅ 10⁵⁴ joules ≈ 2.61 ⋅ 10¹⁰ foes ≈ 26,100,000,000 supernovae

To brute force a 256-bit key requires detonating a quarter of the stars in the 
Milky Way _just to pay the universe’s tax bill on the computation._

This is a silly idea and it is not happening until we find some way to suspend the 
Second Law of Thermodynamics.

### … aren’t you worried about quantum computers?

If it ever becomes possible to build a large-scale quantum computer, it will be
able to brute-force a 256-bit key in only roughly 2¹²⁸ attempts. I invite you to
repeat the classical computer analysis for a quantum computer running 
[Grover’s algorithm](https://en.wikipedia.org/wiki/Grover%27s_algorithm). It is true
it requires much less energy to be liberated as heat, but the levels are still
unimaginably large.

### … does `-s` do nothing?

`pwgen` gave users the option of generating passwords phonetically (by default)
or via a random number generator (`-s`). Phonetic password generation is no longer
a best practice in 2026, and for that reason all `rpass` passwords are generated
with a cryptographically-secure pseudorandom number generator based off the 
well-studied HC-128 stream cipher.

`-s` is included as a command line flag so as to not break existing pipelines that
use `pwgen`, but it’s a no-op.

### … won't `-H` be supported?

`-H` allowed a `pwgen` user to provide a filename (and optionally a seed value),
using that file to generate a SHA-1 hash. That and the seed would be used for 
random-_ish_ number generation. It was the worst of both worlds even pre-Y2K: you
gave up the ease of remembering a phonetic password but didn’t have the entropic
strength of a real CSPRNG-generated password. It was borderline-foolish even back
then, and in 2026 it would be foolish to include it.

`-H` is included as a command line flag so as to not break existing pipelines that
use `pwgen`, but it’s a no-op.

## Keylengths and Entropy

| Desired entropy | Minimum password length |
|:---------------:|:-----------------------:|
| 40              |            7            |
| 64              |           11            |
| 80              |           14            |
| 96              |           16            |
| 128             |           21            |
| 192             |           32            |
| 256             |           43            |

## Entropy guarantee

Each password from `rpass` contains six bits of entropy per glyph. See the
preceding section(s) for warnings about how certain flags can undercut this promise.

## How does it work?

Rust’s `SysRng` random number generator is believed to be cryptographically
secure. However, there are no guarantees made about the underlying implementation
and I’d like a little control over that.

We run our own cryptographically secure pseudorandom number generator
based on the well-studied HC-128 stream cipher. To key it and set the
initialization vector we use the Rust built-in RNG, but after that we’re
running entirely HC-128.

We populate a fixed-size buffer with cryptographically secure pseudorandom
noise, eight bits of entropy per byte. We then base64 the buffer into a UTF-8
string where each glyph has six bits of entropy. Upon generating the glyph
buffer, the byte buffer is zeroized.

Eight is 2³, while six is 2 * 3. By setting our buffer size to 12288 bytes
(2¹² * 3) we can ensure the resulting base64 expansion is 16,384 glyphs of
six bits with no padding. Padding could upset our entropy guarantee, which is why
the buffer is sized how it is.

Once those are dealt with we walk down our 16,384 random glyphs. For
each glyph we check if it meets must-exclude directives: if it’s acceptable it gets
added to the password. If at any step we run out of random glyphs, we generate
another set of 16,384 and continue.

If must-include directives are in effect, those are dealt with last and are not part
of our “six bits of entropy per glyph!” guarantee.

Once the password is assembled it’s printed to `stdout` and flushed. Once
printed, the password is zeroized so as to reduce the forensics traces left in 
memory.

Once all passwords are written, the byte buffer and string of random glyphs are
zeroized. (The byte buffer was already zeroized; doing so on exit is just
belt-and-suspenders engineering.)

## License

Apache 2.0. Share and enjoy!
