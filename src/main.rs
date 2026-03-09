pub mod character_generator;
pub mod cmdline;
pub mod printer;
pub mod terminal;

use character_generator::make_password_generator;
use cmdline::get_count;
use cmdline::parse_command_line;
use printer::make_printer;
use std::io::stdout;
use std::io::Write;
use zeroize::Zeroize;

struct Finalizer(Box<dyn FnMut()>);
impl Drop for Finalizer {
    fn drop(&mut self) {
        self.0();
    }
}

fn main() {
    let args = parse_command_line();

    // The password generator closure uses some sensitive memory which
    // must be safely zeroed on program exit, so whenever we create
    // a generator closure we also create a finalizer for the data in
    // that closure.
    let (mut pwgen, pwfinal) = make_password_generator(args.clone());
    let _guard = Finalizer(Box::new(pwfinal));

    // The printer closure encapsulates details such as terminal width,
    // how many to print on a line, and so forth.
    let mut printer = make_printer(args.clone());

    let mut password = String::new();
    let mut stream = stdout();
    for _ in 0..get_count(&args) {
        pwgen(&mut password);
        printer(&password);
        _ = stream.flush();
        password.zeroize();
    }
}
