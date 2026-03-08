use crate::cmdline;
use base64::engine::{general_purpose, GeneralPurpose};
use base64::{alphabet, Engine};
use rand::{Rng, RngExt, SeedableRng};
use rand_hc::Hc128Rng;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};
use zeroize::Zeroize;

pub static CSPRNG: LazyLock<Mutex<Hc128Rng>> =
    LazyLock::new(|| Mutex::new(Hc128Rng::from_rng(&mut rand::rng())));

#[allow(clippy::cast_possible_truncation)]
pub fn make_password_generator(args: &cmdline::Args) -> (impl FnMut(&mut String), impl FnMut()) {
    let mut satisfy_policies = make_satisfier(args);
    let (mut generator, finalizer) = make_character_generator(args);
    let length = args.length.unwrap_or(8);

    (
        move |password: &mut String| {
            password.zeroize();
            password.truncate(0);
            satisfy_policies(password);
            for _ in 0..(length - (password.chars().count()) as u16) {
                let mut ch = generator();
                password.push(ch);
                ch.zeroize();
            }
        },
        finalizer,
    )
}
fn make_satisfier(args: &cmdline::Args) -> impl FnMut(&mut String) {
    // These are taken directly from pwgen.
    const SYMBOLS: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
    const CAPITALS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const NUMBERS: &str = "0123456789";

    let symbols_length = SYMBOLS.chars().count();
    let capitals_length = CAPITALS.chars().count();
    let numbers_length = NUMBERS.chars().count();
    let ensure_symbols = args.ensure_symbols;
    let ensure_capitals = args.ensure_capitals;
    let ensure_numbers = args.ensure_numbers;

    move |password: &mut String| -> () {
        if ensure_symbols {
            password.push(
                SYMBOLS
                    .chars()
                    .nth(CSPRNG.lock().unwrap().random_range(0..symbols_length))
                    .unwrap_or('+'),
            );
        }
        if ensure_capitals {
            password.push(
                CAPITALS
                    .chars()
                    .nth(CSPRNG.lock().unwrap().random_range(0..capitals_length))
                    .unwrap_or('A'),
            );
        }
        if ensure_numbers {
            password.push(
                NUMBERS
                    .chars()
                    .nth(CSPRNG.lock().unwrap().random_range(0..numbers_length))
                    .unwrap_or('7'),
            );
        }
    }
}

fn make_filter(args: &cmdline::Args) -> impl Fn(&char) -> bool {
    let mut remove_set = HashSet::<char>::new();
    let mut vowels = HashSet::<char>::new();
    let mut ambiguous = HashSet::<char>::new();
    args.remove.chars().for_each(|x| {
        _ = &remove_set.insert(x);
    });
    "01aeiouyAEIOUY".to_string().chars().for_each(|x| {
        _ = vowels.insert(x);
    });
    "B8G6I1l0OQDS5Z2".to_string().chars().for_each(|x| {
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

#[allow(clippy::similar_names)]
fn make_character_generator(args: &cmdline::Args) -> (impl FnMut() -> char, impl FnMut()) {
    const BUFFER_SIZE: usize = 12288;
    const ENGINE: GeneralPurpose =
        GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
    let filter = make_filter(args);
    let random_byte_buffer = [0u8; BUFFER_SIZE];
    let rbb_cell = Rc::new(RefCell::new(random_byte_buffer));
    let rcb_cell = Rc::new(RefCell::new(Vec::<char>::new()));
    let mut rcb_index: usize = 0;

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
                    CSPRNG.lock().unwrap().fill_bytes(&mut *rbb);
                    ENGINE.encode(*rbb).chars().for_each(|x| rcb.push(x));
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
