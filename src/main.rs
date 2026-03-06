pub mod character_generator;
pub mod cmdline;
pub mod printer;
pub mod terminal;

use character_generator::make_password_generator;
use cmdline::get_count;
use cmdline::parse_command_line;
use printer::make_printer;
use terminal::get_words_per_line;
use zeroize::Zeroize;

fn main() {
    let args = parse_command_line();
    let (mut pwgen, mut pwfinal) = make_password_generator(&args);
    let mut printer = make_printer(&args);
    let count = get_count(&args);

    for index in 0..count {
        let mut password = pwgen();
        printer(&password, index);
        password.zeroize();
    }
    pwfinal();
}
