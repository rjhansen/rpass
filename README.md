# rpass

I find myself often needing disposable high-quality passwords, so I
figured I’d let Rust do it for me.

## Usage

`rpass -b BITS -c COUNT` generates `COUNT` passwords, each estimated
to have `BITS` of entropy. By default it prints a single 128-bit
password.

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

## Will you support options to make outputs comply with local policy?

`pwgen` does this, but I won’t. If you ask for a 64-bit password that’s
exactly what you should get. Editing the output to ensure it complies
with absurd rules like “no more than three consonants in a row” would
reduce the strength of the password, not guarantee its strength.
