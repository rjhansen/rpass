use crate::cmdline;
use base64::engine::{general_purpose, GeneralPurpose};
use base64::{alphabet, Engine};
use rand::{seq::SliceRandom, Rng, RngExt, SeedableRng};
use rand_hc::Hc128Rng;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};
use zeroize::Zeroize;

pub static CSPRNG: LazyLock<Mutex<Hc128Rng>> =
    LazyLock::new(|| Mutex::new(Hc128Rng::from_rng(&mut rand::rng())));

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::needless_pass_by_value)]
pub fn make_password_generator(args: cmdline::Args) -> (impl FnMut(&mut String), impl FnMut()) {
    let mut satisfy_policies = make_satisfier(args.clone());
    let (mut generator, finalizer) = make_character_generator(args.clone());
    let length = args.length.unwrap_or(8);
    let mut buf = ['\0'; 1024];

    // this is enforced in cmdline.rs
    debug_assert!(length as usize <= buf.len());

    (
        move |password: &mut String| {
            buf.zeroize();
            password.zeroize();
            password.truncate(0);
            password.reserve((length + 1) as usize);
            for index in 0..length {
                buf[index as usize] = generator();
            }
            satisfy_policies(&mut buf[0..length as usize]);
            for index in 0..length {
                password.push(buf[index as usize]);
            }
            buf.zeroize();
        },
        finalizer,
    )
}

#[allow(clippy::needless_pass_by_value)]
fn make_satisfier(args: cmdline::Args) -> impl FnMut(&mut [char]) {
    // These are taken directly from pwgen.
    let symbols: Vec<char> = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect();
    let capitals: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    let numbers: Vec<char> = "0123456789".chars().collect();

    let ensure_symbols = args.ensure_symbols;
    let ensure_capitals = args.ensure_capitals;
    let ensure_numbers = args.ensure_numbers;

    let mut use_positions = Vec::<usize>::new();
    use_positions.resize(ensure_symbols as usize, 0);

    move |buffer: &mut [char]| -> () {
        use_positions.clear();
        for index in 0..buffer.len() {
            use_positions.push(index);
        }
        {
            let mut random = CSPRNG.lock().unwrap();
            use_positions.shuffle(&mut random);
        }

        let mut pos_iter = use_positions.iter();
        if ensure_symbols {
            let buf_pos = *pos_iter.next().unwrap();
            let char_idx = CSPRNG.lock().unwrap().random_range(0..symbols.len());
            buffer[buf_pos] = symbols[char_idx];
        }
        if ensure_capitals {
            let buf_pos = *pos_iter.next().unwrap();
            let char_idx = CSPRNG.lock().unwrap().random_range(0..capitals.len());
            buffer[buf_pos] = capitals[char_idx];
        }
        if ensure_numbers {
            let buf_pos = *pos_iter.next().unwrap();
            let char_idx = CSPRNG.lock().unwrap().random_range(0..numbers.len());
            buffer[buf_pos] = numbers[char_idx];
        }
        use_positions.zeroize();
    }
}

#[allow(clippy::needless_pass_by_value)]
fn make_filter(args: cmdline::Args) -> impl Fn(&char) -> bool {
    let mut remove_set = HashSet::<char>::new();
    let mut vowels = HashSet::<char>::new();
    let mut ambiguous = HashSet::<char>::new();
    args.remove.chars().for_each(|x| {
        _ = &remove_set.insert(x);
    });
    "aeiouyAEIOUY".to_string().chars().for_each(|x| {
        _ = vowels.insert(x);
    });
    "B8G6I1l0OQDS5Z2".to_string().chars().for_each(|x| {
        _ = ambiguous.insert(x);
    });
    move |ch: &char| -> bool {
        !((args.no_capitals && ch.is_ascii_uppercase())
            || (args.no_numbers && ch.is_ascii_digit())
            || (args.no_vowels && vowels.contains(ch))
            || (args.no_ambiguous && ambiguous.contains(ch))
            || remove_set.contains(ch))
    }
}

#[allow(clippy::similar_names)]
#[allow(clippy::needless_pass_by_value)]
fn make_character_generator(args: cmdline::Args) -> (impl FnMut() -> char, impl FnMut()) {
    const BUFFER_SIZE: usize = 12288;
    const ENGINE: GeneralPurpose =
        GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
    let filter = make_filter(args.clone());
    let random_byte_buffer = [0u8; BUFFER_SIZE];
    let rbb_cell = Rc::new(RefCell::new(random_byte_buffer));
    let rcb_cell = Rc::new(RefCell::new(Vec::with_capacity(BUFFER_SIZE * 4 / 3)));
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
                    let mut tmp_str = ENGINE.encode(*rbb);
                    tmp_str.chars().for_each(|x| rcb.push(x));
                    tmp_str.zeroize();
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
