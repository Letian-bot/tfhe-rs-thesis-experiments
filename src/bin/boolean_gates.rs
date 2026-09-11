//! Section 3.1: correctness of homomorphic Boolean gates.
//! Tests NAND, AND, OR, and XOR on all four Boolean input pairs.

use tfhe::boolean::prelude::*;

fn main() {
    let (client_key, server_key) = gen_keys();
    let cases = [(false, false), (false, true), (true, false), (true, true)];

    let mut total = 0usize;
    let mut correct = 0usize;

    for (a, b) in cases {
        let ca = client_key.encrypt(a);
        let cb = client_key.encrypt(b);

        let tests = [
            ("NAND", server_key.nand(&ca, &cb), !(a && b)),
            ("AND",  server_key.and(&ca, &cb),   a && b),
            ("OR",   server_key.or(&ca, &cb),    a || b),
            ("XOR",  server_key.xor(&ca, &cb),   a ^ b),
        ];

        for (gate, ct, expected) in tests {
            let got = client_key.decrypt(&ct);
            let ok = got == expected;
            total += 1;
            correct += usize::from(ok);
            println!("{gate:4}({a}, {b}) -> {got} (expected {expected}) {}", if ok { "OK" } else { "FAIL" });
        }
    }

    println!("\nCorrect: {correct}/{total}");
    assert_eq!(correct, total);
}
