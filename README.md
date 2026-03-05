# rpass
## _… because random passwords should’t be left to chance._

## Intended Audience

System administrators. This tool probably isn’t useful to regular users.

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
to do their daily business.

With respect and a grateful nod of the head to Ted for `pwgen`, an overhaul is needed.

## Usage

`rpass` is, as far as is reasonable, a drop-in replacement for `pwgen`. These
command-line arguments should look familiar:

```
Generates high-entropy passwords.

Usage: rpass [OPTIONS] [LENGTH] [COUNT]

Arguments:
  [LENGTH]
  [COUNT]

Options:
  -c, --capitalize             ensure one or more capital letters
  -A, --no-capitalize          ensure no capital letters
  -n, --numerals               ensure one or more numbers
  -0, --no-numerals            ensure no numbers
  -y, --symbols                ensure at least one special character
  -r, --remove-chars <REMOVE>  omit these characters [default: ""]
  -s, --secure                 use a cryptographic-strength RNG
  -B, --ambiguous              don't include ambiguous characters
  -v, --no-vowels              don't use any vowels
      --copyright              show copyright notice
  -1, --one-column             display results in one column
  -C, --multicolumn            display results in multiple columns
  -b, --bugs                   where to file bug reports
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
  ```

A few of these flags deserve special comment.

* Flags that ensure the output will conform to content policies will reduce the
  overall password strength by making it more predictable. Account for this and
  choose a longer password if needed.
* `-s` does nothing: `rpass` works securely by default and has no less-safe mode.
* When using `-C`, `rpass` will be reasonably accommodating of terminal sizes but
  not unreasonably so. If your terminal width is set to, say, 2500, calibrate your
  expectations accordingly.
* `pwgen`’s `-H` flag is not supported and **will not be** supported (not by me,
  at least). It was borderline-foolish even when Ted first wrote `pwgen`, and in
  today’s threat environment it’s genuinely foolish.
* `LENGTH` must be in the range [6, 43] inclusive.
* `COUNT` must be in the range [1, 1000] inclusive.

## Output

Each password from `rpass` contains six bits of entropy per glyph. See the
preceding section for warnings about how certain flags can undercut this promise.

## How does it work?

Rust’s default random number generator is believed to be cryptographically
secure. However, I’m unaware of exactly how hard cryptographers have looked
at it, and I suspect with all the architectures it runs on a little
skepticism is warranted that it’s cryptographically secure on all of them.

So, we run our own cryptographically secure pseudorandom number generator
based on the well-studied HC-128 stream cipher. To key it and set the
initialization vector we use the Rust built-in RNG, but after that we’re
running entirely HC-128.

We populate a fixed-size buffer with cryptographically secure pseudorandom
noise, eight bits of entropy per byte. We then base64 the buffer into a UTF-8
string where each glyph has six bits of entropy.

Eight is 2³, while six is 2 * 3. By setting our buffer size to 12288 bytes
(2¹² * 3) we can ensure the resulting base64 expansion is 16,384 glyphs of
six bits with no padding. Padding could upset our entropy guarantee, which is why
the buffer is sized how it is.

We begin generating a password by applying whatever must-include directives are in
effect. If it must include a symbol, a capital letter, a number, or whatever, those
are dealt with first and are not part of our “six bits of entropy per glyph!”
guarantee. Once those are dealt with we walk down our 16,384 random glyphs. For
each glyph we check if it meets must-exclude directives: if it’s acceptable it gets
added to the password. If at any step we run out of random glyphs, we generate
another set of 16,384 and continue.

Once the password is assembled it’s printed to `stdout`. Once printed, the password
is zeroized so as to reduce the forensics traces left in memory.

Once all passwords are written, the buffer and string of random glyphs are
zeroized.

## License

Apache 2.0. Share and enjoy!

## Is it perfect?

Of course not, but it’s pretty good.
