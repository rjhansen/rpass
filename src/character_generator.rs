//! Implements closures that abstract away all the messy details of safely
//! generating passwords.

use base64::engine::{general_purpose, GeneralPurpose};
use base64::{alphabet, Engine};
use rand::{seq::SliceRandom, Rng, RngExt, SeedableRng};
use rand_hc::Hc128Rng;
use std::collections::HashSet;
use zeroize::Zeroize;
use crate::cmdline::parse_command_line;

const B64: GeneralPurpose = GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);

/// Encapsulates all data needed to generate passwords.
pub struct PasswordGenerator {
    password_length: usize,
    csprng: Hc128Rng,
    char_buf: [char; 16384],
    cbindex: usize,
    symbols: Vec<char>,
    capitals: Vec<char>,
    numbers: Vec<char>,
    ensure_symbols: bool,
    ensure_capitals: bool,
    ensure_numbers: bool,
    remove_set: HashSet::<char>,
    vowels: HashSet::<char>,
    ambiguous: HashSet::<char>,
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

impl PasswordGenerator {
    /// Creates a new PasswordGenerator object using data from the command line.
    pub fn new() -> PasswordGenerator {
        let args = parse_command_line();
        let _cb = ['\0'; 16384];
        let mut foo = PasswordGenerator {
            password_length: args.length.unwrap_or(8) as usize,
            csprng: Hc128Rng::from_rng(&mut rand::rng()),
            char_buf: _cb,
            cbindex: 0,
            symbols: "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect(),
            capitals: "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect(),
            numbers: "0123456789".chars().collect(),
            ensure_capitals: args.ensure_capitals,
            ensure_numbers: args.ensure_numbers,
            ensure_symbols: args.ensure_symbols,
            remove_set: HashSet::<char>::new(),
            vowels: HashSet::<char>::new(),
            ambiguous: HashSet::<char>::new(),
            no_capitals: args.no_capitals,
            no_numbers: args.no_numbers,
            no_ambiguous: args.no_ambiguous,
            no_vowels: args.no_vowels
        };
        args.remove.chars().for_each(|x| {
            _ = foo.remove_set.insert(x);
        });
        "aeiouyAEIOUY".to_string().chars().for_each(|x| {
            _ = foo.vowels.insert(x);
        });
        "B8G6I1l0OQDS5Z2".to_string().chars().for_each(|x| {
            _ = foo.ambiguous.insert(x);
        });
        foo.replenish_pool();
        foo
    }

    fn filter(self: &PasswordGenerator, ch: &char) -> bool {
        !((self.no_capitals && ch.is_ascii_uppercase())
            || (self.no_numbers && ch.is_ascii_digit())
            || (self.no_vowels && self.vowels.contains(ch))
            || (self.no_ambiguous && self.ambiguous.contains(ch))
            || self.remove_set.contains(ch))
    }

    fn replenish_pool(self: &mut PasswordGenerator) {
        let mut byte_buf = [0u8; 12288];
        self.char_buf.zeroize();
        self.cbindex = 0;
        self.csprng.fill_bytes(&mut byte_buf);
        let mut tmp_str = B64.encode(byte_buf);
        let mut iter = tmp_str.chars();
        for index in 0..16384 {
            self.char_buf[index] = iter.next().unwrap();
        }
        tmp_str.zeroize();
        byte_buf.zeroize();
    }

    fn satisfy_policies(self: &mut PasswordGenerator, buf: &mut [char]) {
        let mut use_positions = vec![0 as usize; self.password_length];
        use_positions.clear();
        for index in 0..use_positions.len() { use_positions[index] = index; }
        use_positions.shuffle(&mut self.csprng);

        // use_positions[0] = what pos to use for symbol replacement
        // use_positions[1] = what pos to use for capital replacement
        // use_positions[2] = what pos to use for numeric replacement
        //
        // Since use_positions is a Fisher-Yates shuffle of a buffer
        // initially populated as 0..args.length, we're guaranteed these
        // three positions will be distinct and randomly selected.

        if self.ensure_symbols {
            buf[use_positions[0]] = self.symbols[self.csprng.random_range(0..self.symbols.len())];
        }
        if self.ensure_capitals {
            buf[use_positions[1]] = self.capitals[self.csprng.random_range(0..self.capitals.len())];
        }
        if self.ensure_numbers {
            buf[use_positions[2]] = self.numbers[self.csprng.random_range(0..self.numbers.len())];
        }
        use_positions.zeroize();
    }

    /// Generates a single policy conforming to what’s specified on the
    /// command line.
    pub fn generate(&mut self) -> String {
        let mut buf = ['\0'; 64];
        let mut password = String::new();
        password.zeroize();
        password.truncate(0);
        password.reserve(self.password_length);
        for index in 0..self.password_length {
            loop {
                if self.cbindex >= 16384 {
                    self.replenish_pool()
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
            password.push(buf[index as usize]);
        }
        buf.zeroize();
        password
    }
}
