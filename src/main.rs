pub mod cmdline;

use base64::engine::general_purpose;
use base64::{alphabet, engine::GeneralPurpose, Engine as _};
use cmdline::parse_command_line;
use rand::{Rng, RngExt, SeedableRng};
use rand_hc::Hc128Rng;
use std::collections::HashSet;
use terminal_size::{terminal_size, Height, Width};
use zeroize::Zeroize;

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

fn main() {
    let args = parse_command_line();

    const SYMBOLS: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
    const CAPITALS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const BUFFER_SIZE: usize = 1024;
    const ENGINE: GeneralPurpose =
        GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);

    let mut buf = [0u8; BUFFER_SIZE];
    let mut csprng = Hc128Rng::from_rng(&mut rand::rng());
    csprng.fill_bytes(&mut buf);
    let mut pool = ENGINE.encode(&buf);
    buf.zeroize();
    let mut iterator = pool.chars();
    let filter = make_filter(&args);
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
        let mut remaining = length;
        let mut password = "".to_string();

        if args.ensure_symbols {
            password.push(
                SYMBOLS
                    .chars()
                    .nth(csprng.random_range(0..SYMBOLS.chars().count()))
                    .unwrap_or_else(|| '+'),
            );
            remaining -= 1;
        }
        if args.ensure_capitals {
            password.push(
                CAPITALS
                    .chars()
                    .nth(csprng.random_range(0..CAPITALS.chars().count()))
                    .unwrap_or_else(|| 'A'),
            );
            remaining -= 1;
        }
        while remaining > 0 {
            let mut ch = match iterator.next() {
                Some(x) => match filter(&x) {
                    true => x,
                    false => continue,
                },
                None => {
                    pool.zeroize();
                    csprng.fill_bytes(&mut buf);
                    pool = ENGINE.encode(&buf);
                    buf.zeroize();
                    iterator = pool.chars();
                    continue;
                }
            };
            password.push(ch);
            ch.zeroize();
            remaining -= 1;
        }

        if args.multi_column && !args.one_column {
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
    pool.zeroize();
    buf.zeroize();
}
