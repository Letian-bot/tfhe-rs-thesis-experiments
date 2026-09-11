// Reproducing Programmable Bootstrapping (Section 3.3).
//
// Demonstrates that a programmable bootstrap applies a function to a
// ciphertext WITHOUT decrypting it:  Enc(x) --PBS(f)--> Enc(f(x)).
//
// Two functions are tested, both NON-LINEAR (so they genuinely need a
// lookup table / PBS, and could not be done by a cheap linear ciphertext
// operation):
//   1. square:  f(x) = x^2 mod 16   (a clear mathematical function)
//   2. arbitrary lookup table: f defined by an explicit table with no
//      pattern, showing PBS computes ANY table, not just "nice" functions.
//
// Uses the low-level core_crypto API so the lookup-table construction
// (generate_programmable_bootstrap_glwe_lut) and its application
// (programmable_bootstrap_lwe_ciphertext) are both explicit, matching the
// blind-rotation mechanism described in Chapter 2.
//
// Run with: cargo run --release
// Cargo.toml: tfhe = { version = "~1.6.2", features = ["boolean", "shortint", "integer"] }
//   (core_crypto is available; if a name doesn't resolve, send the error.)

use tfhe::core_crypto::prelude::*;

fn main() {
    // Toy parameters (same family as the environment-validation example).
    // NOT security-rated; for functional demonstration only.
    let small_lwe_dimension = LweDimension(742);
    let glwe_dimension = GlweDimension(1);
    let polynomial_size = PolynomialSize(2048);
    let lwe_noise_distribution =
        Gaussian::from_dispersion_parameter(StandardDev(0.000007069849454709433), 0.0);
    let glwe_noise_distribution =
        Gaussian::from_dispersion_parameter(StandardDev(0.00000000000000029403601535432533), 0.0);
    let pbs_base_log = DecompositionBaseLog(23);
    let pbs_level = DecompositionLevelCount(1);
    let ciphertext_modulus = CiphertextModulus::new_native();

    let mut boxed_seeder = new_seeder();
    let seeder = boxed_seeder.as_mut();
    let mut secret_generator =
        SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());
    let mut encryption_generator =
        EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);

    println!("Generating keys...");
    let small_lwe_sk =
        LweSecretKey::generate_new_binary(small_lwe_dimension, &mut secret_generator);
    let glwe_sk =
        GlweSecretKey::generate_new_binary(glwe_dimension, polynomial_size, &mut secret_generator);
    let big_lwe_sk = glwe_sk.clone().into_lwe_secret_key();

    let std_bootstrapping_key = par_allocate_and_generate_new_lwe_bootstrap_key(
        &small_lwe_sk,
        &glwe_sk,
        pbs_base_log,
        pbs_level,
        glwe_noise_distribution,
        ciphertext_modulus,
        &mut encryption_generator,
    );
    let mut fourier_bsk = FourierLweBootstrapKey::new(
        std_bootstrapping_key.input_lwe_dimension(),
        std_bootstrapping_key.glwe_size(),
        std_bootstrapping_key.polynomial_size(),
        std_bootstrapping_key.decomposition_base_log(),
        std_bootstrapping_key.decomposition_level_count(),
    );
    convert_standard_lwe_bootstrap_key_to_fourier(&std_bootstrapping_key, &mut fourier_bsk);
    drop(std_bootstrapping_key);

    // 4-bit message space -> 16 possible values, with 1 bit of padding.
    let message_modulus = 1u64 << 4; // 16
    let delta = (1_u64 << 63) / message_modulus;
    let signed_decomposer =
        SignedDecomposer::new(DecompositionBaseLog(5), DecompositionLevelCount(1));

    // Helper: run PBS applying closure f to an encrypted input value, return decrypted result.
    let mut apply_pbs = |input: u64, f: &dyn Fn(u64) -> u64| -> u64 {
        // Encrypt input under the small LWE key.
        let plaintext = Plaintext(input * delta);
        let lwe_in = allocate_and_encrypt_new_lwe_ciphertext(
            &small_lwe_sk,
            plaintext,
            lwe_noise_distribution,
            ciphertext_modulus,
            &mut encryption_generator,
        );

        // Build the lookup table (accumulator) encoding f.
        let accumulator: GlweCiphertextOwned<u64> = generate_programmable_bootstrap_glwe_lut(
            polynomial_size,
            glwe_dimension.to_glwe_size(),
            message_modulus as usize,
            ciphertext_modulus,
            delta,
            f,
        );

        // Apply the programmable bootstrap: input stays encrypted throughout.
        let mut pbs_out = LweCiphertext::new(
            0u64,
            big_lwe_sk.lwe_dimension().to_lwe_size(),
            ciphertext_modulus,
        );
        programmable_bootstrap_lwe_ciphertext(&lwe_in, &mut pbs_out, &accumulator, &fourier_bsk);

        // Decrypt and decode.
        let out_pt: Plaintext<u64> = decrypt_lwe_ciphertext(&big_lwe_sk, &pbs_out);
        signed_decomposer.closest_representable(out_pt.0) / delta
    };

    // ---- Experiment 1: square, f(x) = x^2 mod 16 ----
    println!("\n--- PBS test 1: square, f(x) = x^2 mod 16 ---");
    let square = |x: u64| (x * x) % 16;
    let mut all_ok = true;
    for x in 0u64..16 {
        let got = apply_pbs(x, &square);
        let expected = square(x);
        let ok = got == expected;
        all_ok &= ok;
        println!(
            "  x={:2}  PBS -> {:2}  expected {:2}  {}",
            x, got, expected, if ok { "OK" } else { "MISMATCH" }
        );
    }
    println!("  square: {}", if all_ok { "ALL OK" } else { "SOME FAILED" });

    // ---- Experiment 2: arbitrary lookup table (no pattern) ----
    println!("\n--- PBS test 2: arbitrary table (no mathematical pattern) ---");
    // One output per input 0..15; deliberately irregular.
    let table: [u64; 16] = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3];
    let lut = |x: u64| table[(x as usize) % 16];
    let mut all_ok2 = true;
    for x in 0u64..16 {
        let got = apply_pbs(x, &lut);
        let expected = lut(x);
        let ok = got == expected;
        all_ok2 &= ok;
        println!(
            "  x={:2}  PBS -> {:2}  expected {:2}  {}",
            x, got, expected, if ok { "OK" } else { "MISMATCH" }
        );
    }
    println!("  table: {}", if all_ok2 { "ALL OK" } else { "SOME FAILED" });

    println!("\nBoth tests apply a function through PBS without ever decrypting the input.");
    println!("A non-linear function and an arbitrary table both work, supporting the");
    println!("Chapter 2 claim that PBS computes any function expressible as a lookup table.");
}
