# rpass

I find myself often needing disposable high-quality passwords, so I
figured I’d let Rust do it for me.

## Usage

`rpass LENGTH COUNT` generates `COUNT` passwords of length `LENGTH`,
with a guaranteed six bits of entropy per letter. Note that the 
options `-c`, `-n`, and/or `-y` will populate these special
characters in such a way as to reduce the entropy below six bits
per glyph.

`rpass --help` will show you more, of course.

## License

Apache 2.0. Share and enjoy!

## Why not use…?

Because no existing tool meets my standards, and a lot of popular
guidance is [genuinely bad](https://www.howtogeek.com/30184/10-ways-to-generate-a-random-password-from-the-command-line/).
(No, you shouldn’t feed `date | md5sum` and use that.)

The best existing tool is `pwgen`, originally written by Ted Ts’o. Ted
certainly knows his stuff, but he’s stopped maintaining the codebase and
some rot has set in. For instance, if `/dev/urandom` isn’t available it
silently falls back to the `random()` call, and that was never meant to
generate cryptographically secure random sequences. It also uses SHA-1
for hashing, and … look, we can do better today, so let’s.

## How does it work?

Rust’s default random number generator is believed to be cryptographically
secure. However, I’m unaware of exactly how hard cryptographers have looked
at it, and I suspect with all the architectures it runs on a little
skepticism is warranted that it’s cryptographically secure on all of them.

So, we run our own cryptographically secure pseudorandom number generator
based on the well-studied HC-128 stream cipher. To key it and set the
initialization vector we use the Rust built-in RNG, but after that we’re
running entirely HC-128.

For each password we generate the requisite number of bits, base64 it,
print it to stdout, then safely zeroize the memory.

## Is it perfect?

Of course not, but it’s pretty good.

## Will you support…

### … inclusion or exclusion of special characters?

It's in there as of 1.2.

### … seeding the RNG with the SHA-1 hash of a file?

No.

### … a cryptographically secure mode?

The `-s` flag is included for compatibility but does nothing. 
`rpass` is not specially hardened against attacks, but does use
a cryptographically secure random number generator.