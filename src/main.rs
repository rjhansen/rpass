pub mod cmdline;

use base64::engine::general_purpose;
use base64::{alphabet, engine::GeneralPurpose, Engine as _};
use cmdline::parse_command_line;
use rand::{Rng, RngExt, SeedableRng};
use rand_hc::Hc128Rng;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
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

    fn make_character_generator(
        filter: &impl Fn(&char) -> bool,
    ) -> (impl FnMut() -> char, impl FnMut() -> ()) {
        const BUFFER_SIZE: usize = 12288;
        const ENGINE: GeneralPurpose =
            GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
        let random_byte_buffer = [0u8; BUFFER_SIZE];
        let rbb_cell = Rc::new(RefCell::new(random_byte_buffer));
        let rcb_cell = Rc::new(RefCell::new(Vec::<char>::new()));
        let mut rcb_index: usize = 0;
        let mut csprng = Hc128Rng::from_rng(&mut rand::rng());

        let rb1 = rbb_cell.clone();
        let rb2 = rbb_cell.clone();
        let rc1 = rcb_cell.clone();
        let rc2 = rcb_cell.clone();

        (
            move || -> char {
                let mut ch: char;
                loop {
                    let mut rbb = rb1.borrow_mut();
                    let mut rcb = rc1.borrow_mut();
                    if rcb_index >= rcb.len() {
                        rbb.zeroize();
                        rcb.zeroize();
                        rcb.clear();
                        rcb_index = 0;
                        csprng.fill_bytes(&mut *rbb);
                        ENGINE
                            .encode(*rbb)
                            .chars()
                            .into_iter()
                            .for_each(|x| rcb.push(x));
                        rbb.zeroize();
                    }
                    ch = rcb[rcb_index];
                    rcb_index += 1;
                    if filter(&ch) {
                        break;
                    }
                }
                ch
            },
            move || -> () {
                let mut rbb = rb2.borrow_mut();
                let mut rcb = rc2.borrow_mut();
                rbb.zeroize();
                rcb.zeroize();
            },
        )
    }

    fn make_satisfier(args: &cmdline::Args) -> impl FnMut(&mut String) -> () {
        // These are taken directly from pwgen.
        const SYMBOLS: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
        let symbols_length = SYMBOLS.chars().count();

        // These are taken directly from pwgen.
        const CAPITALS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let capitals_length = CAPITALS.chars().count();

        // These are taken directly from pwgen.
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

    fn get_terminal_width() -> u16 {
        match terminal_size() {
            Some((Width(w), Height(_))) => {
                if w <= 7 {
                    7
                } else if w < 160 {
                    w
                } else {
                    160
                }
            }
            None => 80,
        }
    }

    let args = parse_command_line();
    let filter = make_filter(&args);
    let mut satisfy_policies = make_satisfier(&args);
    let (mut generator, mut finalizer) = make_character_generator(&filter);
    let mut count = args.count.unwrap_or_else(|| 1);
    let length = args.length.unwrap_or_else(|| 8);
    let line_width = get_terminal_width();
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
    finalizer();
}
