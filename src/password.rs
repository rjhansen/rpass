//! Implements closures that abstract away all the messy details of safely
//! generating passwords.

use crate::cmdline::{Args, MAX_PASSWORD_LENGTH, parse_command_line};
use base64::engine::{GeneralPurpose, general_purpose};
use base64::{Engine, alphabet};
use rand::rngs::SysRng;
use rand::{Rng, RngExt, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha20Rng;
use std::collections::HashSet;
use std::process::exit;
use std::sync::LazyLock;
use zeroize::Zeroize;

const B64: GeneralPurpose = GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
static SYMBOLS: LazyLock<Vec<char>> =
    LazyLock::new(|| "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect());
static CAPITALS: LazyLock<Vec<char>> =
    LazyLock::new(|| "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect());
static NUMBERS: LazyLock<Vec<char>> = LazyLock::new(|| "0123456789".chars().collect());
static VOWELS: LazyLock<HashSet<char>> = LazyLock::new(|| "aeiouAEIOU".chars().collect());
static AMBIGUOUS: LazyLock<HashSet<char>> = LazyLock::new(|| "B8G6I1l0OQDS5Z2".chars().collect());

/// Encapsulates all data needed to generate passwords.
pub struct PasswordGenerator {
    password_length: usize,
    csprng: ChaCha20Rng,
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
            csprng: match ChaCha20Rng::try_from_rng(&mut SysRng) {
                Ok(rng) => rng,
                Err(e) => {
                    eprintln!("error: could not initialize random number generator!");
                    eprintln!("error: this should never happen, but it just did.");
                    eprintln!("error: definitely a bug -- please report it.");
                    eprintln!("error: underlying cause -- {}", e);
                    exit(1);
                }
            },
            char_buf: ['\0'; 0x4000],
            cbindex: 0,
            ensure_capitals: args.ensure_capitals,
            ensure_numbers: args.ensure_numbers,
            ensure_symbols: args.ensure_symbols,
            remove_set: HashSet::new(),
            no_capitals: args.no_capitals,
            no_numbers: args.no_numbers,
            no_ambiguous: args.no_ambiguous,
            no_vowels: args.no_vowels,
        };
        for ch in args.remove.chars() {
            generator.remove_set.insert(ch);
        }
        {
            if generator.ensure_symbols && (&*SYMBOLS).iter().all(|c| !generator.filter(c)) {
                eprintln!("error: symbols are required, but all were excluded");
                exit(1);
            }
            if generator.ensure_numbers && (&*NUMBERS).iter().all(|c| !generator.filter(c)) {
                eprintln!("error: numbers are required, but all were excluded");
                exit(1);
            }
            if generator.ensure_capitals && (&*CAPITALS).iter().all(|c| !generator.filter(c)) {
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
            || (self.no_vowels && VOWELS.contains(ch))
            || (self.no_ambiguous && AMBIGUOUS.contains(ch))
            || self.remove_set.contains(ch))
    }

    fn replenish_pool(&mut self) {
        let mut byte_buf = [0u8; 12288];
        // This should be unnecessary, as we zeroize sensitive memory
        // as it's used. Still, belt and suspenders engineering…
        self.char_buf.zeroize();
        self.cbindex = 0;
        self.csprng.fill_bytes(&mut byte_buf);
        let mut tmp_str = B64.encode(byte_buf);
        byte_buf.zeroize();
        let mut iter = tmp_str.chars();
        for index in 0..0x4000 {
            self.char_buf[index] = match iter.next() {
                Some(ch) => ch,
                None => {
                    eprintln!("error: weird, this should never happen, but it just did.");
                    eprintln!("error: ran out of random data while replenishing pool");
                    eprintln!("error: definitely a bug -- please report it!");
                    exit(1);
                }
            };
        }
        tmp_str.zeroize();
    }

    fn satisfy_policies(&mut self, buf: &mut [char]) {
        let mut indices: Vec<usize> = (0..self.password_length).collect();
        indices.shuffle(&mut self.csprng);

        // indices[0] = what pos to use for symbol replacement
        // indices[1] = what pos to use for capital replacement
        // indices[2] = what pos to use for numeric replacement
        //
        // Since use_positions is a Fisher-Yates shuffle of a buffer
        // initially populated as 0..args.length, we're guaranteed these
        // three positions will be distinct and randomly selected.

        if self.ensure_symbols {
            let cands: Vec<&char> = (&*SYMBOLS).iter().filter(|c| self.filter(c)).collect();
            buf[indices[0]] = *cands[self.csprng.random_range(0..cands.len())];
        }
        if self.ensure_capitals {
            let cands: Vec<&char> = (&*CAPITALS).iter().filter(|c| self.filter(c)).collect();
            buf[indices[1]] = *cands[self.csprng.random_range(0..cands.len())];
        }
        if self.ensure_numbers {
            let cands: Vec<&char> = (&*NUMBERS).iter().filter(|c| self.filter(c)).collect();
            buf[indices[2]] = *cands[self.csprng.random_range(0..cands.len())];
        }
        indices.zeroize();
    }

    /// Generates a single policy conforming to what’s specified on the
    /// command line.
    #[allow(clippy::needless_range_loop)]
    pub fn generate(&mut self) -> String {
        let mut buf = ['\0'; MAX_PASSWORD_LENGTH as usize];
        let mut password = String::new();
        password.reserve(self.password_length);
        for index in 0..self.password_length {
            loop {
                if self.cbindex >= 0x4000 {
                    self.replenish_pool();
                }
                let mut ch = self.char_buf[self.cbindex];
                self.char_buf[self.cbindex].zeroize();
                self.cbindex += 1;
                if self.filter(&ch) {
                    buf[index] = ch;
                    break;
                }
                ch.zeroize();
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
