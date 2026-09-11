//! Section 3.1: repeated NAND correctness check (2,000 trials).
//! Uses DEFAULT_PARAMETERS and a fixed plaintext-input PRNG seed for repeatability.

use tfhe::boolean::prelude::*;

struct XorShift(u64);
impl XorShift {
    fn next_bool(&mut self) -> bool {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 & 1) == 1
    }
}

fn main() {
    const TRIALS: usize = 2000;
    let (client_key, server_key) = gen_keys();
    let mut rng = XorShift(0x243F6A8885A308D3);
    let mut failures = 0usize;

    for i in 0..TRIALS {
        let a = rng.next_bool();
        let b = rng.next_bool();
        let ca = client_key.encrypt(a);
        let cb = client_key.encrypt(b);
        let got = client_key.decrypt(&server_key.nand(&ca, &cb));
        let expected = !(a && b);
        if got != expected {
            failures += 1;
            eprintln!("trial {i}: NAND({a}, {b}) -> {got}, expected {expected}");
        }
    }

    println!("Trials: {TRIALS}");
    println!("Failures: {failures}");
}
