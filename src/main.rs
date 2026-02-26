pub mod cmdline;

use base64::engine::general_purpose;
use base64::{alphabet, engine::GeneralPurpose, Engine as _};
use cmdline::parse_command_line;
use rand::{Rng, SeedableRng};
use rand_hc::Hc128Rng;
use zeroize::Zeroize;

fn main() {
    let args = parse_command_line();
    const ENGINE: GeneralPurpose =
        GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
    let mut buf = vec![0u8; (args.bits as usize) / 8];
    let mut csprng = Hc128Rng::from_rng(&mut rand::rng());

    for _ in 0..args.count {
        csprng.fill_bytes(&mut buf);
        println!("{}", ENGINE.encode(&buf));

        // The password needs to be securely scrubbed in a way the
        // code optimizer won't optimize away. The zeroize crate
        // comes to our rescue.
        buf.zeroize();
    }
}
