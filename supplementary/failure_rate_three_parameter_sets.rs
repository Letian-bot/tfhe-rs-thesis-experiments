// Empirical failure-rate test for TFHE-rs's boolean NAND gate.
//
// For each of the three official parameter sets, generate keys ONCE, then
// run many independent NAND trials with random inputs and freshly sampled
// ciphertexts (reusing the same keys), counting how many trials produce
// the wrong answer. This estimates the empirical failure rate to compare
// against each parameter set's documented failure-probability bound
// (thesis Section 3.3 / Section 3.4).
//
// Run with: cargo run --release
//
// NOTE: this will take a while — roughly (trials_per_set x avg_gate_time)
// per parameter set. With TRIALS = 2000 and the gate times you measured
// earlier (~12-21ms), expect a few minutes total. Lower TRIALS if you want
// a quicker first pass, then raise it once you know it runs cleanly.

use std::time::Instant;
use tfhe::boolean::prelude::*;
use tfhe::boolean::parameters::{
    DEFAULT_PARAMETERS, PARAMETERS_ERROR_PROB_2_POW_MINUS_165, TFHE_LIB_PARAMETERS,
};

// Simple xorshift PRNG for picking random cleartext inputs — this is NOT
// cryptographic randomness, it's only used to pick which of the 4 input
// combinations to test each trial, not for any encryption.
struct XorShift(u64);
impl XorShift {
    fn next_bool(&mut self) -> bool {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 & 1) == 1
    }
}

fn run_failure_rate_test(name: &str, params: &BooleanParameters, trials: u32) {
    println!("\n=== {name} ===");

    let client_key = ClientKey::new(params);
    let server_key = ServerKey::new(&client_key);

    let mut rng = XorShift(0x243F6A8885A308D3 ^ trials as u64); // fixed seed for reproducibility
    let mut failures = 0u32;
    let start = Instant::now();

    for i in 0..trials {
        let a = rng.next_bool();
        let b = rng.next_bool();
        let expected = !(a && b);

        let ct_a = client_key.encrypt(a);
        let ct_b = client_key.encrypt(b);
        let ct_result = server_key.nand(&ct_a, &ct_b);
        let result = client_key.decrypt(&ct_result);

        if result != expected {
            failures += 1;
            println!("  trial {i}: MISMATCH — NAND({a},{b}) = {result}, expected {expected}");
        }
    }

    let elapsed = start.elapsed();
    let failure_rate = failures as f64 / trials as f64;

    println!("Trials: {trials}");
    println!("Failures: {failures}");
    println!("Empirical failure rate: {failure_rate:e}");
    println!("Total time: {elapsed:.2?} ({:.2?} / trial average)", elapsed / trials);
}

pub fn main() {
    let trials: u32 = 2000; // lower this (e.g. 200) for a quicker first pass

    run_failure_rate_test("DEFAULT_PARAMETERS (documented error <= 2^-40)", &DEFAULT_PARAMETERS, trials);
    run_failure_rate_test(
        "PARAMETERS_ERROR_PROB_2_POW_MINUS_165 (documented error <= 2^-165)",
        &PARAMETERS_ERROR_PROB_2_POW_MINUS_165,
        trials,
    );
    run_failure_rate_test("TFHE_LIB_PARAMETERS", &TFHE_LIB_PARAMETERS, trials);

    println!("\nDone. Record trials, failures, and empirical failure rate for each set.");
}
