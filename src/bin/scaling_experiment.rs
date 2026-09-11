// Controlled experiment: bit width x operation type vs homomorphic cost.
//
// Protocol (fixed in advance):
//   - widths n in {8, 16, 32, 64}, each tested for Add and Multiply
//   - Add:      a, b in [0, floor((2^n - 1) / 2)]   => a + b < 2^n, no overflow
//   - Multiply: a, b in [0, floor(sqrt(2^n - 1))]    => a * b < 2^n, no overflow
//   - 3 warm-up runs (not recorded), then 30 recorded runs per condition
//   - fresh random plaintext each run
//   - ONLY the homomorphic operation is timed (keygen, encryption, decryption excluded)
//   - correctness checked every run
//   - reports: correct count, mean operation time (ms), standard deviation (ms)
//
// Run with: cargo run --release
// Cargo.toml: tfhe = { version = "~1.6.2", features = ["boolean", "shortint", "integer"] }

use std::time::Instant;
use tfhe::prelude::*;
use tfhe::{generate_keys, set_server_key, ConfigBuilder, FheUint8, FheUint16, FheUint32, FheUint64};

const WARMUP: usize = 3;
const RUNS: usize = 30;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, bound: u64) -> u64 {
        if bound <= 1 { 0 } else { self.next() % bound }
    }
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}
fn stddev(xs: &[f64], m: f64) -> f64 {
    (xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64).sqrt()
}

fn isqrt(v: u128) -> u128 {
    if v < 2 { return v; }
    let mut x = (v as f64).sqrt() as u128;
    while x * x > v { x -= 1; }
    while (x + 1) * (x + 1) <= v { x += 1; }
    x
}

fn main() {
    println!("Generating keys (once, outside all timing)...");
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    println!("\nwidth, op, correct, mean_ms, sd_ms");

    // $plain  = plaintext integer type (u8/u16/u32/u64)
    // $cipher = ciphertext type (FheUint8/...)
    macro_rules! bench {
        ($bits:expr, $plain:ty, $cipher:ty, $seed:expr) => {{
            let bits: u32 = $bits;
            let add_bound: u64 = (((1u128 << bits) - 1) / 2 + 1) as u64; // exclusive
            let mul_bound: u64 = (isqrt((1u128 << bits) - 1) + 1) as u64; // exclusive

            // ---- Addition ----
            {
                let mut rng = Rng($seed);
                let mut times = Vec::with_capacity(RUNS);
                let mut correct = 0usize;
                for run in 0..(WARMUP + RUNS) {
                    let a = rng.below(add_bound) as $plain;
                    let b = rng.below(add_bound) as $plain;
                    let ea = <$cipher>::encrypt(a, &client_key);
                    let eb = <$cipher>::encrypt(b, &client_key);
                    let t0 = Instant::now();
                    let ec = &ea + &eb;
                    let dt = t0.elapsed().as_secs_f64() * 1000.0;
                    let got: $plain = ec.decrypt(&client_key);
                    if run >= WARMUP {
                        times.push(dt);
                        if got == a.wrapping_add(b) { correct += 1; }
                    }
                }
                let m = mean(&times);
                println!("{}, add, {}/{}, {:.2}, {:.2}", bits, correct, RUNS, m, stddev(&times, m));
            }

            // ---- Multiplication ----
            {
                let mut rng = Rng($seed ^ 0xDEAD_BEEF);
                let mut times = Vec::with_capacity(RUNS);
                let mut correct = 0usize;
                for run in 0..(WARMUP + RUNS) {
                    let a = rng.below(mul_bound) as $plain;
                    let b = rng.below(mul_bound) as $plain;
                    let ea = <$cipher>::encrypt(a, &client_key);
                    let eb = <$cipher>::encrypt(b, &client_key);
                    let t0 = Instant::now();
                    let ec = &ea * &eb;
                    let dt = t0.elapsed().as_secs_f64() * 1000.0;
                    let got: $plain = ec.decrypt(&client_key);
                    if run >= WARMUP {
                        times.push(dt);
                        if got == a.wrapping_mul(b) { correct += 1; }
                    }
                }
                let m = mean(&times);
                println!("{}, mul, {}/{}, {:.2}, {:.2}", bits, correct, RUNS, m, stddev(&times, m));
            }
        }};
    }

    bench!(8u32,  u8,  FheUint8,  0x1111_1111);
    bench!(16u32, u16, FheUint16, 0x2222_2222);
    bench!(32u32, u32, FheUint32, 0x3333_3333);
    bench!(64u32, u64, FheUint64, 0x4444_4444);

    println!("\nDone. Columns: width, operation, correct/{RUNS}, mean time (ms), std dev (ms).");
}
