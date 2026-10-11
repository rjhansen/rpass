use crate::cmdline::{Args, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH, parse_command_line};
use base64::engine::{GeneralPurpose, general_purpose};
use base64::{Engine, alphabet};
use chacha20::ChaCha20Rng;
use rand::rngs::SysRng;
use rand::{Rng, RngExt, SeedableRng, TryRng, seq::SliceRandom};
use std::collections::HashSet;
use std::sync::LazyLock;
use zeroize::{Zeroize, Zeroizing};

const B64: GeneralPurpose = GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
static SYMBOLS: LazyLock<Vec<char>> =
    LazyLock::new(|| "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect());
static CAPITALS: LazyLock<Vec<char>> =
    LazyLock::new(|| "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect());
static NUMBERS: LazyLock<Vec<char>> = LazyLock::new(|| "0123456789".chars().collect());
static VOWELS: LazyLock<HashSet<char>> = LazyLock::new(|| "aeiouAEIOU".chars().collect());
static AMBIGUOUS: LazyLock<HashSet<char>> = LazyLock::new(|| "B8G6I1l0OQDS5Z2".chars().collect());
const MIN_ENTROPY_BITS: u32 = 6 * MIN_PASSWORD_LENGTH as u32;

/// Returns the shortest password length that reaches `MIN_ENTROPY_BITS`
/// when each glyph is drawn uniformly from `alphabet_size` candidates.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn min_length_for(alphabet_size: usize) -> usize {
    (((MIN_ENTROPY_BITS as f32) / (alphabet_size as f32).log2()).ceil()) as usize
}

/// Which glyphs the user has ruled out.
struct Exclusions {
    no_capitals: bool,
    no_numbers: bool,
    no_vowels: bool,
    no_ambiguous: bool,
    removed: HashSet<char>,
}

impl Exclusions {
    fn from_args(args: &Args) -> Self {
        Self {
            no_capitals: args.no_capitals,
            no_numbers: args.no_numbers,
            no_vowels: args.no_vowels,
            no_ambiguous: args.no_ambiguous,
            removed: args.remove.chars().collect(),
        }
    }

    /// Returns whether `ch` may appear in a password.
    fn allows(&self, ch: &char) -> bool {
        !((self.no_capitals && ch.is_ascii_uppercase())
            || (self.no_numbers && ch.is_ascii_digit())
            || (self.no_vowels && VOWELS.contains(ch))
            || (self.no_ambiguous && AMBIGUOUS.contains(ch))
            || self.removed.contains(ch))
    }

    /// Returns the members of `candidates` that may appear in a password.
    fn allowed_from(&self, candidates: &[char]) -> Vec<char> {
        candidates
            .iter()
            .copied()
            .filter(|ch| self.allows(ch))
            .collect()
    }
}

/// Encapsulates all data needed to generate passwords.
pub struct PasswordGenerator {
    password_length: usize,
    csprng: ChaCha20Rng,
    char_buf: [char; 0x4000],
    cbindex: usize,
    ensure_symbols: bool,
    ensure_capitals: bool,
    ensure_numbers: bool,
    exclusions: Exclusions,
    valid_symbols: Vec<char>,
    valid_capitals: Vec<char>,
    valid_numbers: Vec<char>,
}

/// Seeds a ChaCha20 CSPRNG from the system RNG. The seed is the CSPRNG's
/// key, so it's held in a `Zeroizing` wrapper that wipes it on every way
/// out of this function. The CSPRNG zeroizes its own state on drop
/// (`chacha20`'s `zeroize` feature).
fn new_csprng() -> Result<ChaCha20Rng, String> {
    let mut seed = Zeroizing::new([0u8; 32]);
    if let Err(e) = SysRng.try_fill_bytes(seed.as_mut_slice()) {
        return Err(format!(
            "Could not initialize random number generator! This should never happen,
but it just did. This is definitely a bug -- please report it.

Underlying cause: {e}"
        ));
    }
    Ok(ChaCha20Rng::from_seed(*seed))
}

impl Drop for PasswordGenerator {
    fn drop(&mut self) {
        self.char_buf.zeroize();
        self.cbindex = 0;
    }
}

impl PasswordGenerator {
    /// Creates a new PasswordGenerator object using data from the command line.
    pub fn new_from_args(args: &Args) -> Result<Self, String> {
        let exclusions = Exclusions::from_args(args);

        let mut generator = Self {
            password_length: args.length.unwrap_or(8) as usize,
            csprng: new_csprng()?,
            char_buf: ['\0'; 0x4000],
            cbindex: 0,
            ensure_capitals: args.ensure_capitals,
            ensure_numbers: args.ensure_numbers,
            ensure_symbols: args.ensure_symbols,
            valid_symbols: exclusions.allowed_from(&SYMBOLS),
            valid_capitals: exclusions.allowed_from(&CAPITALS),
            valid_numbers: exclusions.allowed_from(&NUMBERS),
            exclusions,
        };

        if generator.ensure_symbols && generator.valid_symbols.is_empty() {
            return Err("symbols are required, but all were excluded".into());
        }
        if generator.ensure_numbers && generator.valid_numbers.is_empty() {
            return Err("numbers are required, but all were excluded".into());
        }
        if generator.ensure_capitals && generator.valid_capitals.is_empty() {
            return Err("capitals are required, but all were excluded".into());
        }
        let b64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let alphabet_size = b64_chars
            .chars()
            .filter(|c| generator.exclusions.allows(c))
            .count();
        if alphabet_size == 0 {
            return Err("you seem to be excluding the entire set of base64 characters".into());
        }
        if min_length_for(alphabet_size) > generator.password_length {
            return Err(
                "the reduced alphabet, combined with this size of generated password, would
fail to meet the entropy guarantee of 36 bits."
                    .into(),
            );
        }
        generator.replenish_pool();
        Ok(generator)
    }

    /// Creates a new PasswordGenerator object from the process's command line.
    pub fn new() -> Result<Self, String> {
        let args = parse_command_line();
        Self::new_from_args(args)
    }

    fn replenish_pool(&mut self) {
        // Zeroizing wipes these on drop, so they're cleaned up even if we
        // leave this function by panicking.
        let mut byte_buf = Zeroizing::new([0u8; 12288]);
        // This should be unnecessary, as we zeroize sensitive memory
        // as it's used. Still, belt and suspenders engineering…
        self.char_buf.zeroize();
        self.cbindex = 0;
        self.csprng.fill_bytes(byte_buf.as_mut_slice());
        // Encode from a slice: passing the array itself would hand `encode`
        // a copy that never gets zeroized.
        let tmp_str = Zeroizing::new(B64.encode(byte_buf.as_slice()));
        // 12288 bytes always encode to exactly 0x4000 unpadded base64 glyphs.
        for (slot, byte) in self.char_buf.iter_mut().zip(tmp_str.bytes()) {
            *slot = char::from(byte);
        }
    }

    fn satisfy_policies(&mut self, buf: &mut [char]) {
        let mut indices: Zeroizing<Vec<usize>> =
            Zeroizing::new((0..self.password_length).collect());
        indices.shuffle(&mut self.csprng);

        // indices[0] = what pos to use for symbol replacement
        // indices[1] = what pos to use for capital replacement
        // indices[2] = what pos to use for numeric replacement
        //
        // Since use_positions is a Fisher-Yates shuffle of a buffer
        // initially populated as 0..args.length, we're guaranteed these
        // three positions will be distinct and randomly selected.

        if self.ensure_symbols {
            buf[indices[0]] =
                self.valid_symbols[self.csprng.random_range(0..self.valid_symbols.len())];
        }
        if self.ensure_capitals {
            buf[indices[1]] =
                self.valid_capitals[self.csprng.random_range(0..self.valid_capitals.len())];
        }
        if self.ensure_numbers {
            buf[indices[2]] =
                self.valid_numbers[self.csprng.random_range(0..self.valid_numbers.len())];
        }
    }

    /// Generates a single policy conforming to what’s specified on the
    /// command line.
    #[allow(clippy::needless_range_loop)]
    pub fn generate(&mut self) -> Zeroizing<String> {
        let mut buf = Zeroizing::new(['\0'; MAX_PASSWORD_LENGTH as usize]);
        let mut password = Zeroizing::new(String::with_capacity(self.password_length));
        for index in 0..self.password_length {
            loop {
                if self.cbindex >= 0x4000 {
                    self.replenish_pool();
                }
                let mut ch = self.char_buf[self.cbindex];
                self.char_buf[self.cbindex].zeroize();
                self.cbindex += 1;
                if self.exclusions.allows(&ch) {
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
        password
    }
}
