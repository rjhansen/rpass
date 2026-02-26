# rpass

I find myself often needing disposable high-quality
passwords, so I figured I'd let Rust do it for me.

## Usage
`rpass -b BITS -c COUNT` generates `COUNT` passwords, each 
estimated to have `BITS` of entropy. By default it prints
a single 128-bit password.

`rpass --help` will show you more, of course.

## License
Apache 2.0. Share and enjoy!


