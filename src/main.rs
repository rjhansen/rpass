use base64::{engine::general_purpose, Engine as _};
use rand::{Rng, SeedableRng};
use rand_hc::Hc128Rng;

fn main() {
    let mut buf = [0; 16];
    Hc128Rng::from_rng(&mut rand::rng()).fill_bytes(&mut buf);
    println!("{}", general_purpose::STANDARD.encode(&buf));
}
