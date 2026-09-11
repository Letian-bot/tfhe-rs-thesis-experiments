# Reproducing and Benchmarking TFHE via TFHE-rs

This repository contains the Rust code and experimental results used in the MSc dissertation:

**Reproducing and Benchmarking TFHE via TFHE-rs: A Study of Fully Homomorphic Encryption**

University of Glasgow, 2026.

The experiments reproduce several core behaviours of TFHE using TFHE-rs and evaluate the computational cost of homomorphic operations.

## Experiments

The repository contains the experiments described in Chapter 3 of the dissertation.

| Section | Program | Description |
|---|---|---|
| 3.1 Reproducing Homomorphic Operations | `boolean_gates.rs` | Tests homomorphic NAND, AND, OR and XOR gates on all four Boolean input pairs (16 cases in total). |
| 3.1 Repeated Correctness Test | `nand_failure_rate.rs` | Performs 2,000 NAND evaluations using randomly generated Boolean inputs and checks the decrypted outputs. |
| 3.2 Evaluating Circuit Depth | `depth_experiment.rs` | Evaluates chained `NAND(ct, ct)` operations at circuit depths from 1 to 10,000 and records correctness and execution time. |
| 3.3 Reproducing Programmable Bootstrapping | `pbs_experiment.rs` | Demonstrates programmable bootstrapping using `x^2 mod 16` and an arbitrary lookup table over a 4-bit message space. |
| 3.4 Evaluating Multi-bit Homomorphic Computation | `scaling_experiment.rs` | Benchmarks homomorphic addition and multiplication for 8-, 16-, 32- and 64-bit encrypted unsigned integers. |

## Repository Structure

```text
.
├── Cargo.toml
├── README.md
├── src/
│   └── bin/
│       ├── boolean_gates.rs
│       ├── nand_failure_rate.rs
│       ├── depth_experiment.rs
│       ├── pbs_experiment.rs
│       └── scaling_experiment.rs
└── results/
    └── thesis_results.md
```

## Requirements

- Rust
- Cargo
- TFHE-rs 1.6.x
- Release build mode

The project uses the following TFHE-rs dependency:

```toml
tfhe = { version = "~1.6.2", features = ["boolean", "shortint", "integer"] }
```

Release mode should be used because homomorphic operations are significantly slower in debug builds.

## Running the Experiments

Clone the repository:

```bash
git clone https://github.com/Letian-bot/tfhe-rs-thesis-experiments.git
cd tfhe-rs-thesis-experiments
```

Run an individual experiment with:

```bash
cargo run --release --bin boolean_gates
cargo run --release --bin nand_failure_rate
cargo run --release --bin depth_experiment
cargo run --release --bin pbs_experiment
cargo run --release --bin scaling_experiment
```

Some experiments, particularly the larger circuit-depth and integer multiplication experiments, may take longer to complete.

## Experimental Results

The main results reported in the dissertation include:

- All 16 Boolean gate test cases produced the expected results.
- No incorrect outputs were observed in the 2,000-trial NAND correctness test.
- Correct computation was maintained through a chain of 10,000 NAND gates.
- Programmable bootstrapping produced the expected output for all tested inputs for both the non-linear function and the arbitrary lookup table.
- All recorded multi-bit addition and multiplication operations produced the expected results.

For the multi-bit benchmark, the mean execution times reported in the dissertation were:

| Width | Addition Mean | Multiplication Mean |
|---:|---:|---:|
| 8 bit | 79.68 ms | 156.84 ms |
| 16 bit | 105.35 ms | 452.42 ms |
| 32 bit | 183.90 ms | 1731.75 ms |
| 64 bit | 399.13 ms | 7255.85 ms |

The complete results corresponding to Chapter 3 are available in:

`results/thesis_results.md`

## Benchmark Methodology

For the multi-bit benchmark, four integer widths were tested: 8, 16, 32 and 64 bits.

Addition and multiplication were evaluated separately. Input ranges were restricted so that the plaintext result did not overflow the corresponding unsigned integer type.

For each experimental condition:

- cryptographic keys were generated once before measurement;
- 3 warm-up operations were performed and excluded;
- 30 runs were recorded using randomly generated plaintext inputs;
- only the homomorphic operation was timed;
- key generation, encryption and decryption were excluded from the measured execution time;
- each decrypted result was checked for correctness;
- mean execution time and standard deviation were calculated.

## Reproducibility Notes

Execution times depend on hardware, operating system, TFHE-rs version and other environmental factors. The absolute timings should therefore not be interpreted as universal TFHE-rs performance values.

The benchmark results in this repository are intended to reproduce the experimental evaluation reported in the dissertation and to demonstrate the observed performance trends between different integer widths and homomorphic operations.
