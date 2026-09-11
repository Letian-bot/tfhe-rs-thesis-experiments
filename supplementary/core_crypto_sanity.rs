// Starter example using TFHE-rs's low-level core_crypto API.
// Adapted from Zama's official core_crypto tutorial:
// https://docs.zama.org/tfhe-rs/references/core-crypto-api/tutorial
//
// This computes 2 * 3 two ways: a cheap "cleartext multiplication" (linear,
// no bootstrap) and a Programmable Bootstrap (PBS), which is the mechanism
// you'll adapt into a NAND-gate lookup table for the thesis reproduction.
//
// Run with: cargo run --release
// (release mode matters a lot for FHE — debug mode is extremely slow)

use tfhe::core_crypto::prelude::*;

pub fn main() {
    // DISCLAIMER: these toy parameters are for learning/testing only —
    // they are NOT guaranteed to be secure or always correct. Swap in one
    // of TFHE-rs's audited parameter sets for anything you report numbers on.
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

    // CSPRNGs for key generation and encryption noise
    let mut boxed_seeder = new_seeder();
    let seeder = boxed_seeder.as_mut();
    let mut secret_generator = SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());
    let mut encryption_generator =
        EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);

    println!("Generating keys...");

    // LWE secret key (used to encrypt the input) and GLWE secret key (used
    // to build the bootstrapping key / perform blind rotation)
    let small_lwe_sk = LweSecretKey::generate_new_binary(small_lwe_dimension, &mut secret_generator);
    let glwe_sk =
        GlweSecretKey::generate_new_binary(glwe_dimension, polynomial_size, &mut secret_generator);
    let big_lwe_sk = glwe_sk.clone().into_lwe_secret_key();

    // Bootstrapping key: a GGSW-style encryption of the LWE key's bits under
    // the GLWE key (see Chapter 2, Section "The TFHE Scheme")
    let std_bootstrapping_key = par_allocate_and_generate_new_lwe_bootstrap_key(
        &small_lwe_sk,
        &glwe_sk,
        pbs_base_log,
        pbs_level,
        glwe_noise_distribution,
        ciphertext_modulus,
        &mut encryption_generator,
    );

    // Convert to the Fourier domain for fast blind rotation
    let mut fourier_bsk = FourierLweBootstrapKey::new(
        std_bootstrapping_key.input_lwe_dimension(),
        std_bootstrapping_key.glwe_size(),
        std_bootstrapping_key.polynomial_size(),
        std_bootstrapping_key.decomposition_base_log(),
        std_bootstrapping_key.decomposition_level_count(),
    );
    convert_standard_lwe_bootstrap_key_to_fourier(&std_bootstrapping_key, &mut fourier_bsk);
    drop(std_bootstrapping_key);

    // 4-bit message space, encoded with one bit of padding (standard TFHE encoding)
    let message_modulus = 1u64 << 4;
    let input_message = 3u64;
    let delta = (1_u64 << 63) / message_modulus;
    let plaintext = Plaintext(input_message * delta);

    let lwe_ciphertext_in: LweCiphertextOwned<u64> = allocate_and_encrypt_new_lwe_ciphertext(
        &small_lwe_sk,
        plaintext,
        lwe_noise_distribution,
        ciphertext_modulus,
        &mut encryption_generator,
    );

    // --- Method 1: cleartext multiplication (linear, cheap, no bootstrap) ---
    let mut cleartext_mul_ct = lwe_ciphertext_in.clone();
    println!("Performing cleartext multiplication...");
    lwe_ciphertext_cleartext_mul(&mut cleartext_mul_ct, &lwe_ciphertext_in, Cleartext(2));

    let cleartext_mul_plaintext: Plaintext<u64> =
        decrypt_lwe_ciphertext(&small_lwe_sk, &cleartext_mul_ct);

    let signed_decomposer = SignedDecomposer::new(DecompositionBaseLog(5), DecompositionLevelCount(1));
    let cleartext_mul_result: u64 =
        signed_decomposer.closest_representable(cleartext_mul_plaintext.0) / delta;

    println!("Checking result...");
    assert_eq!(6, cleartext_mul_result);
    println!("Cleartext multiplication result is correct! Expected 6, got {cleartext_mul_result}");

    // --- Method 2: Programmable Bootstrap (PBS) ---
    // This is the mechanism to adapt: replace `|x: u64| 2 * x` with a NAND
    // truth table over a packed two-bit input to reproduce the gate from
    // Chapter 3 of the thesis.
    let accumulator: GlweCiphertextOwned<u64> = generate_programmable_bootstrap_glwe_lut(
        polynomial_size,
        glwe_dimension.to_glwe_size(),
        message_modulus as usize,
        ciphertext_modulus,
        delta,
        |x: u64| 2 * x,
    );

    let mut pbs_mul_ct = LweCiphertext::new(0u64, big_lwe_sk.lwe_dimension().to_lwe_size(), ciphertext_modulus);
    println!("Computing PBS...");
    programmable_bootstrap_lwe_ciphertext(&lwe_ciphertext_in, &mut pbs_mul_ct, &accumulator, &fourier_bsk);

    let pbs_mul_plaintext: Plaintext<u64> = decrypt_lwe_ciphertext(&big_lwe_sk, &pbs_mul_ct);
    let pbs_mul_result: u64 = signed_decomposer.closest_representable(pbs_mul_plaintext.0) / delta;

    println!("Checking result...");
    assert_eq!(6, pbs_mul_result);
    println!("Multiplication via PBS result is correct! Expected 6, got {pbs_mul_result}");
}
