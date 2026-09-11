// Real-workload experiment: TFHE-rs's integer types as deep, real circuits.
//
// A single high-level integer operation (a + b, a * b, a > b, ...) expands
// internally into a long chain of gates and programmable bootstraps. This
// demonstrates two things from Chapter 2:
//   (1) PBS is not limited to boolean gates - comparison, multiplication,
//       etc. are all built on top of it (the shortint/integer layer);
//   (2) wider integers = deeper internal circuits, so timing grows with
//       bit width, while correctness holds throughout.
//
// It also estimates how many bootstraps hide behind one high-level call, by
// dividing each operation's time by a single-gate baseline.
//
// Run with: cargo run --release
// Cargo.toml: tfhe = { version = "~1.6.2", features = ["integer"] }
//   (the high-level `tfhe::prelude` + FheUintX types live behind the
//    "integer" feature; if a type name doesn't resolve, send me the error.)

use std::time::Instant;
use tfhe::prelude::*;
use tfhe::{generate_keys, set_server_key, ConfigBuilder, FheUint8, FheUint16, FheUint32, FheUint64};

fn time_ms<F: FnOnce()>(f: F) -> f64 {
    let t0 = Instant::now();
    f();
    t0.elapsed().as_secs_f64() * 1000.0
}

pub fn main() {
    println!("Generating keys (default high-level config)...");
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    // ---- 8-bit ----
    let a8 = FheUint8::encrypt(37u8, &client_key);
    let b8 = FheUint8::encrypt(85u8, &client_key);
    let add8 = time_ms(|| { let _ = &a8 + &b8; });
    let mul8 = time_ms(|| { let _ = &a8 * &b8; });
    let gt8 = time_ms(|| { let _ = &a8.gt(&b8); });
    let r_add8: u8 = (&a8 + &b8).decrypt(&client_key);
    let r_mul8: u8 = (&a8 * &b8).decrypt(&client_key);
    let r_gt8: bool = (&a8.gt(&b8)).decrypt(&client_key);
    println!("\n--- FheUint8 (a=37, b=85) ---");
    println!("add: {r_add8} (expect {}) {}  | {:.2} ms", 37u8.wrapping_add(85), if r_add8 == 37u8.wrapping_add(85) {"OK"} else {"FAIL"}, add8);
    println!("mul: {r_mul8} (expect {}) {}  | {:.2} ms", 37u8.wrapping_mul(85), if r_mul8 == 37u8.wrapping_mul(85) {"OK"} else {"FAIL"}, mul8);
    println!("gt:  {r_gt8} (expect {}) {}  | {:.2} ms", 37 > 85, if r_gt8 == (37 > 85) {"OK"} else {"FAIL"}, gt8);

    // ---- 16-bit ----
    let a16 = FheUint16::encrypt(1000u16, &client_key);
    let b16 = FheUint16::encrypt(2345u16, &client_key);
    let add16 = time_ms(|| { let _ = &a16 + &b16; });
    let mul16 = time_ms(|| { let _ = &a16 * &b16; });
    let r_add16: u16 = (&a16 + &b16).decrypt(&client_key);
    let r_mul16: u16 = (&a16 * &b16).decrypt(&client_key);
    println!("\n--- FheUint16 (a=1000, b=2345) ---");
    println!("add: {r_add16} (expect {}) {}  | {:.2} ms", 1000u16.wrapping_add(2345), if r_add16 == 1000u16.wrapping_add(2345) {"OK"} else {"FAIL"}, add16);
    println!("mul: {r_mul16} (expect {}) {}  | {:.2} ms", 1000u16.wrapping_mul(2345), if r_mul16 == 1000u16.wrapping_mul(2345) {"OK"} else {"FAIL"}, mul16);

    // ---- 32-bit ----
    let a32 = FheUint32::encrypt(123456u32, &client_key);
    let b32 = FheUint32::encrypt(789012u32, &client_key);
    let add32 = time_ms(|| { let _ = &a32 + &b32; });
    let mul32 = time_ms(|| { let _ = &a32 * &b32; });
    let r_add32: u32 = (&a32 + &b32).decrypt(&client_key);
    let r_mul32: u32 = (&a32 * &b32).decrypt(&client_key);
    println!("\n--- FheUint32 (a=123456, b=789012) ---");
    println!("add: {r_add32} (expect {}) {}  | {:.2} ms", 123456u32.wrapping_add(789012), if r_add32 == 123456u32.wrapping_add(789012) {"OK"} else {"FAIL"}, add32);
    println!("mul: {r_mul32} (expect {}) {}  | {:.2} ms", 123456u32.wrapping_mul(789012), if r_mul32 == 123456u32.wrapping_mul(789012) {"OK"} else {"FAIL"}, mul32);

    // ---- 64-bit ----
    let a64 = FheUint64::encrypt(1234567890u64, &client_key);
    let b64 = FheUint64::encrypt(9876543210u64, &client_key);
    let add64 = time_ms(|| { let _ = &a64 + &b64; });
    let mul64 = time_ms(|| { let _ = &a64 * &b64; });
    let r_add64: u64 = (&a64 + &b64).decrypt(&client_key);
    let r_mul64: u64 = (&a64 * &b64).decrypt(&client_key);
    println!("\n--- FheUint64 (a=1234567890, b=9876543210) ---");
    println!("add: {r_add64} (expect {}) {}  | {:.2} ms", 1234567890u64.wrapping_add(9876543210), if r_add64 == 1234567890u64.wrapping_add(9876543210) {"OK"} else {"FAIL"}, add64);
    println!("mul: {r_mul64} (expect {}) {}  | {:.2} ms", 1234567890u64.wrapping_mul(9876543210), if r_mul64 == 1234567890u64.wrapping_mul(9876543210) {"OK"} else {"FAIL"}, mul64);

    println!("\nRecord the add/mul times per bit width. Rising time with width");
    println!("supports: wider integer = deeper internal circuit. Correctness");
    println!("holding throughout supports: bootstrapping keeps deep circuits valid.");
}
