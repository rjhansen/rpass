pub mod character_generator;
pub mod cmdline;
pub mod printer;
pub mod terminal;

use character_generator::make_password_generator;
use cmdline::get_count;
use cmdline::parse_command_line;
use printer::make_printer;
use zeroize::Zeroize;

fn main() {
    let args = parse_command_line();
    if args.secure {
        eprintln!(
r#"info: the -s flag is unnecessary. pwgen would by default create passwords
      from phonemes, but by passing -s it would abandon phonemes in favor
      of high-entropy random glyphs. rpass only generates high-entropy
      random glyphs. You may safely drop this flag from your pipeline."#);
    }

    // The password generator closure uses some sensitive memory which
    // must be safely zeroed on program exit, so whenever we create
    // a generator closure we also create a finalizer for the data in
    // that closure.
    let (mut pwgen, mut pwfinal) = make_password_generator(&args);

    // The printer closure encapsulates details such as terminal width,
    // how many to print on a line, and so forth.
    let mut printer = make_printer(&args);

    for index in 0..get_count(&args) {
        let mut password = pwgen();
        printer(&password, index);
        password.zeroize();
    }
    pwfinal();
}
