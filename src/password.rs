//! Implements closures that abstract away all the messy details of safely
//! generating passwords.

use crate::cmdline::{Args, parse_command_line};
use base64::engine::{general_purpose, GeneralPurpose};
use base64::{alphabet, Engine};
use rand::rngs::SysRng;
use rand::{seq::SliceRandom, Rng, RngExt, SeedableRng};
use rand_hc::Hc128Rng;
use std::collections::HashSet;
use std::process::exit;
use std::sync::{LazyLock, Mutex};
use zeroize::Zeroize;

const B64: GeneralPurpose = GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
static SYMBOLS: LazyLock<Mutex<Vec<char>>> = LazyLock::new(|| {
    let sym: Vec<char> = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect();
    Mutex::new(sym)
});
static CAPITALS: LazyLock<Mutex<Vec<char>>> = LazyLock::new(|| {
    let caps: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    Mutex::new(caps)
});
static NUMBERS: LazyLock<Mutex<Vec<char>>> = LazyLock::new(|| {
    let nums: Vec<char> = "0123456789".chars().collect();
    Mutex::new(nums)
});
static VOWELS: LazyLock<Mutex<HashSet<char>>> = LazyLock::new(|| {
    let vowels: HashSet<char> = "aeiouAEIOU".chars().collect();
    Mutex::new(vowels)
});
static AMBIGUOUS: LazyLock<Mutex<HashSet<char>>> = LazyLock::new(|| {
    let ambiguous: HashSet<char> = "B8G6I1l0OQDS5Z2".chars().collect();
    Mutex::new(ambiguous)
});

/// Encapsulates all data needed to generate passwords.
pub struct PasswordGenerator {
    password_length: usize,
    csprng: Hc128Rng,
    char_buf: [char; 0x4000],
    cbindex: usize,
    ensure_symbols: bool,
    ensure_capitals: bool,
    ensure_numbers: bool,
    remove_set: HashSet<char>,
    no_capitals: bool,
    no_numbers: bool,
    no_ambiguous: bool,
    no_vowels: bool,
}

impl Drop for PasswordGenerator {
    fn drop(&mut self) {
        self.char_buf.zeroize();
        self.cbindex = 0;
    }
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordGenerator {
    /// Creates a new PasswordGenerator object using data from the command line.
    pub fn new_from_args(args: &Args) -> Self {
        let mut generator = Self {
            password_length: args.length.unwrap_or(8) as usize,
            csprng: Hc128Rng::try_from_rng(&mut SysRng).unwrap_or_else(|_| {
                    eprintln!("error: could not initialize random number generator!");
                    exit(1);
                }),
            char_buf: ['\0'; 0x4000],
            cbindex: 0,
            ensure_capitals: args.ensure_capitals,
            ensure_numbers: args.ensure_numbers,
            ensure_symbols: args.ensure_symbols,
            remove_set: HashSet::new(),
            no_capitals: args.no_capitals,
            no_numbers: args.no_numbers,
            no_ambiguous: args.no_ambiguous,
            no_vowels: args.no_vowels
        };
        for ch in args.remove.chars() {
            generator.remove_set.insert(ch);
        }
        {
            let syms = SYMBOLS.lock().expect("mutex poisoned");
            let nums = NUMBERS.lock().expect("mutex poisoned");
            let caps = CAPITALS.lock().expect("mutex poisoned");
            if generator.ensure_symbols && syms.iter().all(|c| !generator.filter(c)) {
                eprintln!("error: symbols are required, but all were excluded");
                exit(1);
            }
            if generator.ensure_numbers && nums.iter().all(|c| !generator.filter(c)) {
                eprintln!("error: numbers are required, but all were excluded");
                exit(1);
            }
            if generator.ensure_capitals && caps.iter().all(|c| !generator.filter(c)) {
                eprintln!("error: capitals are required, but all were excluded");
                exit(1);
            }
        }
        let b64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        if b64_chars.chars().all(|c| !generator.filter(&c)) {
            eprintln!("error: you seem to be excluding the entire set of base64 characters");
            exit(1);
        }
        generator.replenish_pool();
        generator
    }

    pub fn new() -> Self {
        let args = parse_command_line();
        Self::new_from_args(args)
    }

    fn filter(&self, ch: &char) -> bool {
        !((self.no_capitals && ch.is_ascii_uppercase())
            || (self.no_numbers && ch.is_ascii_digit())
            || (self.no_vowels && VOWELS.lock().expect("mutex poisoned").contains(ch))
            || (self.no_ambiguous && AMBIGUOUS.lock().expect("mutex poisoned").contains(ch))
            || self.remove_set.contains(ch))
    }

    fn replenish_pool(&mut self) {
        let mut byte_buf = [0u8; 12288];
        self.char_buf.zeroize();
        self.cbindex = 0;
        self.csprng.fill_bytes(&mut byte_buf);
        let mut tmp_str = B64.encode(byte_buf);
        let mut iter = tmp_str.chars();
        for index in 0..0x4000 {
            self.char_buf[index] = iter.next().expect("char buffer exhausted");
        }
        tmp_str.zeroize();
        byte_buf.zeroize();
    }

    fn satisfy_policies(&mut self, buf: &mut [char]) {
        let mut indices: Vec<usize> = (0..self.password_length).collect();
        indices.shuffle(&mut self.csprng);

        // use_positions[0] = what pos to use for symbol replacement
        // use_positions[1] = what pos to use for capital replacement
        // use_positions[2] = what pos to use for numeric replacement
        //
        // Since use_positions is a Fisher-Yates shuffle of a buffer
        // initially populated as 0..args.length, we're guaranteed these
        // three positions will be distinct and randomly selected.

        if self.ensure_symbols {
            let symvec = SYMBOLS.lock().expect("mutex poisoned");
            let cands: Vec<&char> = symvec.iter().filter(|c| self.filter(c)).collect();
            buf[indices[0]] = *cands[self.csprng.random_range(0..cands.len())];
        }
        if self.ensure_capitals {
            let capvec = CAPITALS.lock().expect("mutex poisoned");
            let cands: Vec<&char> = capvec.iter().filter(|c| self.filter(c)).collect();
            buf[indices[1]] = *cands[self.csprng.random_range(0..cands.len())];
        }
        if self.ensure_numbers {
            let numvec = NUMBERS.lock().expect("mutex poisoned");
            let cands: Vec<&char> = numvec.iter().filter(|c| self.filter(c)).collect();
            buf[indices[2]] = *cands[self.csprng.random_range(0..cands.len())];
        }
        indices.zeroize();
    }

    /// Generates a single policy conforming to what’s specified on the
    /// command line.
    #[allow(clippy::needless_range_loop)]
    pub fn generate(&mut self) -> String {
        let mut buf = ['\0'; 43];
        let mut password = String::new();
        password.reserve(self.password_length);
        for index in 0..self.password_length {
            loop {
                if self.cbindex >= 0x4000 {
                    self.replenish_pool();
                }
                let ch = self.char_buf[self.cbindex];
                self.cbindex += 1;
                if self.filter(&ch) {
                    buf[index] = ch;
                    break;
                }
            }
        }
        self.satisfy_policies(&mut buf[0..self.password_length]);
        for index in 0..self.password_length {
            password.push(buf[index]);
        }
        buf.zeroize();
        password
    }
}
