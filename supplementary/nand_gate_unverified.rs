// Homomorphic NAND gate via TFHE-rs core_crypto, built on top of the PBS
// mechanism demonstrated in the earlier double-a-number example.
//
// Design: encrypt two bits x1, x2 separately, homomorphically combine them
// into a single ciphertext encoding v = x1 + 2*x2 (a cheap linear op, no
// bootstrap), then drive a Programmable Bootstrap with a lookup table that
// maps each of the four possible values of v to the correct NAND output.
//
// NOTE: this has not been compiled/run yet — I don't have a Rust toolchain
// available on my end to verify it against your exact tfhe crate version.
// If a function name doesn't match (compiler will say "cannot find function
// `X` in this scope" or similar), send me the error and we'll fix it fast.
//
// Run with: cargo run --release

use tfhe::core_crypto::prelude::*;

pub fn main() {
    // Toy parameters — same as the earlier example, NOT security-rated.
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
    let mut secret_generator = SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());
    let mut encryption_generator =
        EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);

    println!("Generating keys...");
    let small_lwe_sk = LweSecretKey::generate_new_binary(small_lwe_dimension, &mut secret_generator);
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

    // Combined-value message space: v = x1 + 2*x2 takes 4 values (2 bits),
    // plus the usual 1 bit of padding -> message_modulus = 4.
    let message_modulus = 4u64;
    let delta = (1_u64 << 63) / message_modulus;
    let signed_decomposer = SignedDecomposer::new(DecompositionBaseLog(5), DecompositionLevelCount(1));

    // Helper: encrypt a single bit under small_lwe_sk with the v-encoding.
    let encrypt_bit = |bit: u64, gen: &mut EncryptionRandomGenerator<DefaultRandomGenerator>| {
        let plaintext = Plaintext(bit * delta);
        allocate_and_encrypt_new_lwe_ciphertext(
            &small_lwe_sk,
            plaintext,
            lwe_noise_distribution,
            ciphertext_modulus,
            gen,
        )
    };

    // NAND lookup table over the combined value v in {0,1,2,3}.
    let nand_accumulator: GlweCiphertextOwned<u64> = generate_programmable_bootstrap_glwe_lut(
        polynomial_size,
        glwe_dimension.to_glwe_size(),
        message_modulus as usize,
        ciphertext_modulus,
        delta,
        |v: u64| if v == 3 { 0 } else { 1 },
    );

    // Evaluate the homomorphic NAND gate on encrypted inputs (x1, x2).
    let homomorphic_nand = |x1: u64,
                             x2: u64,
                             gen: &mut EncryptionRandomGenerator<DefaultRandomGenerator>|
     -> u64 {
        let ct1 = encrypt_bit(x1, gen);
        let ct2 = encrypt_bit(x2, gen);

        // v = x1 + 2*x2 (linear combination, no bootstrap needed here)
        let mut scaled_ct2 = ct2.clone();
        lwe_ciphertext_cleartext_mul(&mut scaled_ct2, &ct2, Cleartext(2));

        let mut combined = ct1.clone();
        lwe_ciphertext_add_assign(&mut combined, &scaled_ct2);

        // Bootstrap combined ciphertext through the NAND lookup table.
        let mut result_ct = LweCiphertext::new(
            0u64,
            big_lwe_sk.lwe_dimension().to_lwe_size(),
            ciphertext_modulus,
        );
        programmable_bootstrap_lwe_ciphertext(&combined, &mut result_ct, &nand_accumulator, &fourier_bsk);

        // Decrypt and decode.
        let result_plaintext: Plaintext<u64> = decrypt_lwe_ciphertext(&big_lwe_sk, &result_ct);
        signed_decomposer.closest_representable(result_plaintext.0) / delta
    };

    println!("Testing homomorphic NAND gate against all four input combinations...\n");

    let cases = [(0u64, 0u64), (0, 1), (1, 0), (1, 1)];
    let mut all_correct = true;

    for (x1, x2) in cases {
        let expected = 1 - (x1 & x2); // cleartext NAND
        let got = homomorphic_nand(x1, x2, &mut encryption_generator);
        let ok = got == expected;
        all_correct &= ok;
        println!(
            "NAND({x1}, {x2}) = {got}  (expected {expected})  {}",
            if ok { "OK" } else { "MISMATCH" }
        );
    }

    println!();
    if all_correct {
        println!("All four input combinations correct — homomorphic NAND gate reproduced successfully.");
    } else {
        println!("Some combinations were incorrect — check parameters / noise budget.");
    }
}
