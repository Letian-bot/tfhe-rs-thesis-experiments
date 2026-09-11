# Reproducing and Benchmarking TFHE via TFHE-rs — Experiments

Rust code for the experiments in Chapter 3 of the dissertation **“Reproducing and Benchmarking TFHE via TFHE-rs: A Study of Fully Homomorphic Encryption”** (University of Glasgow, MSc dissertation draft, September 2026).

The repository is intentionally small: the main binaries correspond directly to Sections 3.1–3.4 of the dissertation. Earlier exploratory code is kept separately under `supplementary/` and is not required to reproduce the reported Chapter 3 experiments.

## Experiments

| Dissertation section | Binary | Purpose |
|---|---|---|
| 3.1 Reproducing Homomorphic Operations | `boolean_gates` | Test NAND, AND, OR and XOR on all four Boolean input pairs (16 cases). |
| 3.1 Repeated correctness check | `nand_failure_rate` | Run 2,000 independently encrypted NAND evaluations and count incorrect outputs. |
| 3.2 Evaluating Circuit Depth | `depth_experiment` | Chain `NAND(ct, ct)` at depths 1–10,000 and measure correctness and runtime. |
| 3.3 Reproducing Programmable Bootstrapping | `pbs_experiment` | Use low-level `core_crypto` PBS for `x^2 mod 16` and an arbitrary lookup table. |
| 3.4 Evaluating Multi-bit Homomorphic Computation | `scaling_experiment` | Benchmark encrypted addition and multiplication for 8/16/32/64-bit integers. |

## Requirements

- Rust / Cargo
- TFHE-rs 1.6.x
- Release mode (FHE code is much slower in debug builds)

The working experiment log records TFHE-rs 1.6.3, Rust 1.97.1, Windows x64, and an Intel Core i7-10870H with 16 GB RAM. The Cargo dependency is pinned to the compatible `~1.6.2` range used by the experiment sources.

## Running the experiments

```bash
cargo run --release --bin boolean_gates
cargo run --release --bin nand_failure_rate
cargo run --release --bin depth_experiment
cargo run --release --bin pbs_experiment
cargo run --release --bin scaling_experiment
```

The depth and scaling experiments can take several minutes. The dissertation run of the 10,000-gate depth condition took about 149 seconds on the reference machine; 64-bit homomorphic multiplication averaged about 7.26 seconds in the controlled scaling benchmark.

## Reported results

The dissertation draft reports:

- all 16 Boolean truth-table cases correct;
- no incorrect NAND outputs in the reported 2,000-trial repeated check;
- correct output through a chain of 10,000 bootstrapped NAND gates, with roughly constant time per gate for the longer chains;
- all 16 inputs correct for both programmable-bootstrap lookup functions;
- all 240 recorded multi-bit arithmetic results correct;
- mean addition time increasing from 79.68 ms (8-bit) to 399.13 ms (64-bit);
- mean multiplication time increasing from 156.84 ms (8-bit) to 7255.85 ms (64-bit).

Exact tables transcribed from Chapter 3 are in [`results/thesis_results.md`](results/thesis_results.md). The original working log is retained as [`results/experiment_log.md`](results/experiment_log.md).

## Repository layout

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
├── results/
│   ├── thesis_results.md
│   └── experiment_log.md
└── supplementary/
    ├── core_crypto_sanity.rs
    ├── nand_gate_unverified.rs
    ├── integer_experiment_early.rs
    └── failure_rate_three_parameter_sets.rs
```

## Notes on reproducibility

Some low-level PBS examples use toy parameters for functional demonstration and are **not security-rated**. They should not be interpreted as recommended production parameters.

The `supplementary/` directory contains development-stage experiments that are useful for tracing the project history but are not needed for the Chapter 3 results. In particular, `nand_gate_unverified.rs` was explicitly marked as not compiled/run in the original source notes.

Runtime values are machine- and version-dependent. When reproducing the benchmarks, record the hardware, operating system, Rust version, TFHE-rs version, build mode, and trial count. The trends are more meaningful than treating the absolute timings as universal TFHE performance figures.

## Status

This repository was assembled from the dissertation experiment sources and aligned with the current dissertation draft. The package has not been recompiled in the packaging environment, so a clean `cargo build --release` should be performed before tagging a final archival release.
