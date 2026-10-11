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
            return Err(format!(
                "the reduced alphabet, combined with this size of generated password, would
fail to meet the entropy guarantee of {MIN_ENTROPY_BITS} bits."
            ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    const BASE64_GLYPHS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    /// Builds an `Args` the way the command line would, without touching
    /// the process-wide one.
    fn args(flags: &[&str]) -> Args {
        Args::parse_from(std::iter::once("rpass").chain(flags.iter().copied()))
    }

    /// Generates `count` passwords for the given command line.
    fn passwords(flags: &[&str], count: usize) -> Vec<String> {
        let mut generator = match PasswordGenerator::new_from_args(&args(flags)) {
            Ok(generator) => generator,
            Err(msg) => panic!("{flags:?} should be accepted, but got: {msg}"),
        };
        std::iter::repeat_with(|| generator.generate().to_string())
            .take(count)
            .collect()
    }

    /// Returns the error message the given command line is refused with.
    fn refusal(flags: &[&str]) -> String {
        match PasswordGenerator::new_from_args(&args(flags)) {
            Ok(_) => panic!("{flags:?} should be refused, but was accepted"),
            Err(msg) => msg,
        }
    }

    fn is_symbol(ch: char) -> bool {
        SYMBOLS.contains(&ch)
    }

    // ---- min_length_for ------------------------------------------------

    #[test]
    fn full_alphabet_needs_the_minimum_length() {
        assert_eq!(
            min_length_for(64),
            MIN_PASSWORD_LENGTH as usize,
            "64 glyphs at six bits each should need exactly the minimum length"
        );
    }

    #[test]
    fn smaller_alphabets_need_longer_passwords() {
        // (alphabet size, shortest length n with size^n >= 2^36)
        for (size, needed) in [
            (63, 7),
            (38, 7),
            (32, 8),
            (22, 9),
            (16, 9),
            (8, 12),
            (6, 14),
            (4, 18),
            (2, 36),
        ] {
            assert_eq!(
                min_length_for(size),
                needed,
                "wrong minimum length for an alphabet of {size}"
            );
        }
    }

    #[test]
    fn one_glyph_alphabet_can_never_be_long_enough() {
        assert!(
            min_length_for(1) > MAX_PASSWORD_LENGTH as usize,
            "a one-glyph alphabet has no entropy, so no length should satisfy it"
        );
    }

    // ---- Exclusions ----------------------------------------------------

    #[test]
    fn nothing_is_excluded_by_default() {
        let exclusions = Exclusions::from_args(&args(&[]));
        for ch in BASE64_GLYPHS.chars().chain(SYMBOLS.iter().copied()) {
            assert!(
                exclusions.allows(&ch),
                "{ch:?} should be allowed by default"
            );
        }
    }

    #[test]
    fn each_flag_excludes_exactly_its_own_glyphs() {
        let cases: [(&[&str], &str); 5] = [
            (&["-A"], "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            (&["-0"], "0123456789"),
            (&["-v"], "aeiouAEIOU"),
            (&["-B"], "B8G6I1l0OQDS5Z2"),
            (&["-r", "xyz+/"], "xyz+/"),
        ];
        for (flags, excluded) in cases {
            let exclusions = Exclusions::from_args(&args(flags));
            for ch in BASE64_GLYPHS.chars() {
                assert_eq!(
                    exclusions.allows(&ch),
                    !excluded.contains(ch),
                    "{flags:?} gave the wrong answer for {ch:?}"
                );
            }
        }
    }

    #[test]
    fn allowed_from_keeps_order_and_drops_only_excluded_glyphs() {
        let exclusions = Exclusions::from_args(&args(&["-B"]));
        let kept: String = exclusions.allowed_from(&NUMBERS).into_iter().collect();
        assert_eq!(kept, "3479", "-B should leave only the unambiguous digits");
    }

    // ---- PasswordGenerator: what it produces ---------------------------

    #[test]
    fn default_length_is_eight() {
        for password in passwords(&[], 50) {
            assert_eq!(password.len(), 8, "{password:?} isn't eight glyphs long");
        }
    }

    #[test]
    fn requested_length_is_honored_at_both_ends_of_the_range() {
        for length in [MIN_PASSWORD_LENGTH, 20, MAX_PASSWORD_LENGTH] {
            let length_arg = length.to_string();
            for password in passwords(&[length_arg.as_str()], 50) {
                assert_eq!(
                    password.len(),
                    usize::from(length),
                    "{password:?} isn't {length} glyphs long"
                );
            }
        }
    }

    #[test]
    fn default_passwords_use_only_base64_glyphs() {
        for password in passwords(&["43"], 200) {
            for ch in password.chars() {
                assert!(
                    BASE64_GLYPHS.contains(ch),
                    "{ch:?} in {password:?} isn't a base64 glyph"
                );
            }
        }
    }

    #[test]
    fn every_base64_glyph_can_appear() {
        // 43,000 glyphs; the chance of any one of the 64 never turning
        // up is about 64 * (63/64)^43000, which is nil.
        let seen: HashSet<char> = passwords(&["43"], 1000)
            .iter()
            .flat_map(|password| password.chars())
            .collect();
        for ch in BASE64_GLYPHS.chars() {
            assert!(seen.contains(&ch), "{ch:?} never appeared");
        }
    }

    #[test]
    fn pool_refills_cleanly() {
        // 43,000 glyphs is several pools' worth (a pool holds 0x4000), so
        // this crosses the refill boundary more than once. A glyph left
        // behind by a bad refill would show up as a NUL.
        for password in passwords(&["43"], 1000) {
            assert_eq!(password.len(), 43, "{password:?} isn't 43 glyphs long");
            assert!(
                !password.contains('\0'),
                "{password:?} contains a NUL from an unfilled pool slot"
            );
        }
    }

    #[test]
    fn excluded_glyphs_never_appear() {
        let cases: [(&[&str], &str); 6] = [
            (&["-A", "30"], "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            (&["-0", "30"], "0123456789"),
            (&["-v", "30"], "aeiouAEIOU"),
            (&["-B", "30"], "B8G6I1l0OQDS5Z2"),
            (&["-r", "xyz+/", "30"], "xyz+/"),
            (
                &["-A", "-0", "-v", "-B", "-r", "bcd", "30"],
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789aeioulbcd",
            ),
        ];
        for (flags, excluded) in cases {
            for password in passwords(flags, 500) {
                for ch in password.chars() {
                    assert!(
                        !excluded.contains(ch),
                        "{flags:?} produced {password:?}, which contains {ch:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn required_classes_are_always_present() {
        for password in passwords(&["-c", "12"], 500) {
            assert!(
                password.chars().any(|ch| ch.is_ascii_uppercase()),
                "-c produced {password:?}, which has no capital"
            );
        }
        for password in passwords(&["-n", "12"], 500) {
            assert!(
                password.chars().any(|ch| ch.is_ascii_digit()),
                "-n produced {password:?}, which has no digit"
            );
        }
        for password in passwords(&["-y", "12"], 500) {
            assert!(
                password.chars().any(is_symbol),
                "-y produced {password:?}, which has no symbol"
            );
        }
    }

    #[test]
    fn all_three_required_classes_fit_in_the_shortest_password() {
        // The three replacements must land on distinct positions, or one
        // would overwrite another. The shortest password is the tightest
        // squeeze. `+` and `/` are removed so that the only symbol in each
        // password is the one that was deliberately placed.
        for password in passwords(&["-c", "-n", "-y", "-r", "+/", "7"], 2000) {
            assert!(
                password.chars().any(|ch| ch.is_ascii_uppercase()),
                "{password:?} has no capital"
            );
            assert!(
                password.chars().any(|ch| ch.is_ascii_digit()),
                "{password:?} has no digit"
            );
            assert_eq!(
                password.chars().filter(|ch| is_symbol(*ch)).count(),
                1,
                "{password:?} should have exactly one symbol"
            );
        }
    }

    #[test]
    fn required_glyphs_respect_exclusions() {
        for password in passwords(&["-c", "-B", "20"], 500) {
            assert!(
                password.chars().any(|ch| ch.is_ascii_uppercase()),
                "-c -B produced {password:?}, which has no capital"
            );
        }
        for password in passwords(&["-n", "-B", "20"], 500) {
            let digits: String = password.chars().filter(char::is_ascii_digit).collect();
            assert!(
                !digits.is_empty(),
                "-n -B produced {password:?}, which has no digit"
            );
            assert!(
                digits.chars().all(|ch| "3479".contains(ch)),
                "-n -B produced {password:?}, which has an ambiguous digit"
            );
        }
        // With every symbol but `!` removed, `!` is the only one -y can use.
        let all_but_bang = "\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
        for password in passwords(&["-y", "-r", all_but_bang, "20"], 500) {
            let symbols: String = password.chars().filter(|ch| is_symbol(*ch)).collect();
            assert_eq!(symbols, "!", "{password:?} should contain exactly one `!`");
        }
    }

    // ---- PasswordGenerator: what it refuses ----------------------------

    #[test]
    fn requiring_a_fully_excluded_class_is_refused() {
        let all_symbols: String = SYMBOLS.iter().collect();
        assert_eq!(
            refusal(&["-y", "-r", all_symbols.as_str(), "12"]),
            "symbols are required, but all were excluded",
            "wrong refusal for -y with every symbol removed"
        );
        assert_eq!(
            refusal(&["-n", "-r", "0123456789", "12"]),
            "numbers are required, but all were excluded",
            "wrong refusal for -n with every digit removed"
        );
        assert_eq!(
            refusal(&["-c", "-r", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "12"]),
            "capitals are required, but all were excluded",
            "wrong refusal for -c with every capital removed"
        );
    }

    #[test]
    fn excluding_every_glyph_is_refused() {
        assert_eq!(
            refusal(&["-r", BASE64_GLYPHS, "43"]),
            "you seem to be excluding the entire set of base64 characters",
            "wrong refusal for removing the whole alphabet"
        );
    }

    #[test]
    fn entropy_floor_is_enforced_at_its_boundaries() {
        let floor = format!("entropy guarantee of {MIN_ENTROPY_BITS} bits");
        // (flags leaving N glyphs, longest refused length, shortest accepted)
        let cases: [(&[&str], u16, u16); 4] = [
            (&["-A"], 6, 7),                                               // 38 glyphs
            (&["-A", "-0", "-v", "-B"], 8, 9),                             // 22 glyphs
            (&["-A", "-0", "-v", "-B", "-r", "bcdfghjkmnpqrst+"], 13, 14), // 6 glyphs
            (&["-A", "-0", "-v", "-r", "bcdfghjklmnpqrstvwx+/"], 35, 36),  // 2 glyphs
        ];
        for (flags, too_short, long_enough) in cases {
            let too_short = too_short.to_string();
            let long_enough = long_enough.to_string();

            let mut refused = flags.to_vec();
            refused.push(too_short.as_str());
            let message = refusal(&refused);
            assert!(
                message.contains(&floor),
                "{refused:?} was refused with the wrong message: {message}"
            );

            let mut accepted = flags.to_vec();
            accepted.push(long_enough.as_str());
            assert_eq!(
                passwords(&accepted, 1).len(),
                1,
                "{accepted:?} should produce a password"
            );
        }
    }

    #[test]
    fn one_glyph_alphabet_is_refused_at_any_length() {
        let message = refusal(&["-A", "-0", "-v", "-r", "bcdfghjklmnpqrstvwxy+/", "43"]);
        assert!(
            message.contains("entropy guarantee"),
            "a one-glyph alphabet was refused with the wrong message: {message}"
        );
    }

    #[test]
    fn unfiltered_minimum_length_is_accepted() {
        assert_eq!(
            passwords(&["6"], 1).len(),
            1,
            "the shortest length with nothing excluded should be accepted"
        );
    }
}
