// Depth experiment: does bootstrapping actually remove the depth ceiling?
//
// Chains NAND gates so the output of one gate becomes the input of the next,
// building a circuit of controllable depth. Because TFHE bootstraps after
// every gate, correctness should hold at ANY depth, and total time should
// grow LINEARLY with depth (each gate costs a fixed amount). If this were a
// leveled scheme without per-gate bootstrapping, deep chains would fail.
//
// Circuit used: ct = NAND(ct, ct), which equals NOT ct. So after d gates,
// the correct value is: start XOR (d mod 2). This is fully predictable at
// every depth, never gets stuck at a constant, and every gate is a real
// bootstrap (unlike NOT, which in TFHE-rs is a cheap linear op).
//
// Run with: cargo run --release
// Cargo.toml: tfhe = { version = "~1.6.2", features = ["boolean"] }

use std::time::Instant;
use tfhe::boolean::prelude::*;

fn run_depth(client_key: &ClientKey, server_key: &ServerKey, start: bool, depth: u32) {
    let mut ct = client_key.encrypt(start);

    let t0 = Instant::now();
    for _ in 0..depth {
        ct = server_key.nand(&ct, &ct); // NAND(x,x) = NOT x
    }
    let elapsed = t0.elapsed();

    let result = client_key.decrypt(&ct);
    // NOT applied d times: flips the bit iff d is odd
    let expected = if depth % 2 == 0 { start } else { !start };
    let ok = result == expected;

    let per_gate = if depth > 0 {
        elapsed.as_secs_f64() * 1000.0 / depth as f64
    } else {
        0.0
    };

    println!(
        "depth {:>7}: result {:<5} expected {:<5} {}  |  total {:>10.2?}  per-gate {:.3} ms",
        depth,
        result,
        expected,
        if ok { "OK" } else { "FAIL" },
        elapsed,
        per_gate,
    );
}

pub fn main() {
    println!("Generating keys (DEFAULT_PARAMETERS)...");
    let (client_key, server_key) = gen_keys();

    println!("\nChaining NAND(ct, ct) = NOT ct, starting from true:\n");

    // A few orders of magnitude. 10000 takes a couple of minutes; adjust if needed.
    let depths = [1u32, 10, 100, 1_000, 10_000];
    for &d in &depths {
        run_depth(&client_key, &server_key, true, d);
    }

    println!("\nIf every row is OK and per-gate time is roughly constant across");
    println!("depths, that supports: bootstrapping removes the depth ceiling, and");
    println!("each gate costs a fixed amount (total time linear in depth).");
    println!("\nOptional: uncomment the line below for a 100000-depth extreme run.");
    // run_depth(&client_key, &server_key, true, 100_000);
}
