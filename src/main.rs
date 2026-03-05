pub mod cmdline;

use base64::engine::general_purpose;
use base64::{alphabet, engine::GeneralPurpose, Engine as _};
use cmdline::parse_command_line;
use rand::{Rng, RngExt, SeedableRng};
use rand_hc::Hc128Rng;
use std::collections::HashSet;
use terminal_size::{terminal_size, Height, Width};
use zeroize::Zeroize;


fn main() {
    fn make_filter(args: &cmdline::Args) -> impl Fn(&char) -> bool {
        let mut remove_set = HashSet::<char>::new();
        let mut vowels = HashSet::<char>::new();
        let mut ambiguous = HashSet::<char>::new();
        args.remove.chars().for_each(|x| {
            _ = &remove_set.insert(x);
        });
        "01aeiouyAEIOUY"
            .to_string()
            .chars()
            .into_iter()
            .for_each(|x| {
                _ = vowels.insert(x);
            });
        "B8G6I1l0OQDS5Z2"
            .to_string()
            .chars()
            .into_iter()
            .for_each(|x| {
                _ = ambiguous.insert(x);
            });
        move |ch: &char| -> bool {
            !(args.no_capitals && ch.is_ascii_uppercase())
                || (args.no_numbers && ch.is_ascii_digit())
                || (args.no_vowels && vowels.contains(ch))
                || (args.no_ambiguous && ambiguous.contains(ch))
                || remove_set.contains(ch)
        }
    }

    fn make_character_generator(filter: &impl Fn(&char) -> bool) -> impl FnMut() -> char {
        const BUFFER_SIZE: usize = 8192;
        const ENGINE: GeneralPurpose =
            GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
        let mut random_byte_buffer = [0u8; BUFFER_SIZE];
        let mut random_char_buffer = Vec::<char>::new();
        let mut random_char_index: usize = 0;
        let mut csprng = Hc128Rng::from_rng(&mut rand::rng());

        move || -> char {
            let mut ch: char;
            loop {
                if random_char_index >= random_char_buffer.len() {
                    random_char_buffer.zeroize();
                    random_char_index = 0;
                    csprng.fill_bytes(&mut random_byte_buffer);
                    ENGINE
                        .encode(random_byte_buffer)
                        .chars()
                        .into_iter()
                        .for_each(|x| random_char_buffer.push(x));
                    random_byte_buffer.zeroize();
                }
                ch = random_char_buffer[random_char_index];
                random_char_index += 1;
                if filter(&ch) {
                    break;
                }
            }
            ch
        }
    }

    fn make_satisfier(args: &cmdline::Args) -> impl FnMut(&mut String) -> () {
        const SYMBOLS: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
        let symbols_length = SYMBOLS.chars().count();
        const CAPITALS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let capitals_length = CAPITALS.chars().count();
        const NUMBERS: &str = "0123456789";
        let numbers_length = NUMBERS.chars().count();

        let ensure_symbols: bool = args.ensure_symbols;
        let ensure_capitals: bool = args.ensure_capitals;
        let ensure_numbers: bool = args.ensure_numbers;
        let mut csprng = Hc128Rng::from_rng(&mut rand::rng());

        move |password: &mut String| -> () {
            if ensure_symbols {
                password.push(
                    SYMBOLS
                        .chars()
                        .nth(csprng.random_range(0..symbols_length))
                        .unwrap_or_else(|| '+'),
                );
            }
            if ensure_capitals {
                password.push(
                    CAPITALS
                        .chars()
                        .nth(csprng.random_range(0..capitals_length))
                        .unwrap_or_else(|| 'A'),
                );
            }
            if ensure_numbers {
                password.push(
                    NUMBERS
                        .chars()
                        .nth(csprng.random_range(0..numbers_length))
                        .unwrap_or_else(|| '7'),
                );
            }
        }
    }

    let args = parse_command_line();
    let filter = make_filter(&args);
    let mut generator = make_character_generator(&filter);
    let mut satisfy_policies = make_satisfier(&args);
    let mut count: u16 = args.count.unwrap_or_else(|| 1);
    let length: u16 = args.length.unwrap_or_else(|| 8);
    let line_width = match terminal_size() {
        Some((Width(w), Height(_))) => {
            if w < 160 {
                w
            } else {
                160
            }
        }
        None => 80,
    };
    let words_per_line = ((line_width as f32 / (length + 1) as f32).ceil() as u16) - 2;
    let mut remaining_in_this_line = words_per_line;

    if args.count.is_none() && args.multi_column {
        count = (words_per_line + 1) * 20;
    }

    for index in 0..count {
        let mut password = "".to_string();
        satisfy_policies(&mut password);
        for _ in 0..(length - (password.chars().count()) as u16) {
            let mut ch = generator();
            password.push(ch);
            ch.zeroize();
        }

        if args.multi_column {
            if remaining_in_this_line > 0 {
                if index < (count - 1) {
                    print!("{} ", password);
                } else {
                    println!("{}", password);
                }
                remaining_in_this_line -= 1;
            } else {
                println!("{}", password);
                remaining_in_this_line = words_per_line;
            }
        } else {
            println!("{}", password);
        }
        password.zeroize();
    }
}
