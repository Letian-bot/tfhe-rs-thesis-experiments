# Results reported in the dissertation draft

These values are transcribed from Chapter 3 of the September 2026 dissertation draft.

## Section 3.1 — Boolean operations

| Gate | Cases | Correct |
|---|---:|---:|
| NAND | 4 | 4 |
| AND | 4 | 4 |
| OR | 4 | 4 |
| XOR | 4 | 4 |

Repeated NAND check: 0 incorrect outputs observed in 2,000 trials.

## Section 3.2 — Circuit depth

| Depth | Correct | Total time | Time/gate |
|---:|:---:|---:|---:|
| 1 | Yes | 17.91 ms | 17.91 ms |
| 10 | Yes | 168.23 ms | 16.82 ms |
| 100 | Yes | 1.54 s | 15.36 ms |
| 1,000 | Yes | 15.63 s | 15.63 ms |
| 10,000 | Yes | 149.21 s | 14.92 ms |

## Section 3.3 — Programmable bootstrapping

All 16 inputs matched for both `f(x) = x^2 mod 16` and the arbitrary 16-entry lookup table.

## Section 3.4 — Multi-bit homomorphic computation

| Width | Add mean | Add SD | Multiply mean | Multiply SD |
|---:|---:|---:|---:|---:|
| 8 | 79.68 ms | 7.29 ms | 156.84 ms | 7.62 ms |
| 16 | 105.35 ms | 6.87 ms | 452.42 ms | 11.01 ms |
| 32 | 183.90 ms | 4.84 ms | 1731.75 ms | 28.41 ms |
| 64 | 399.13 ms | 9.74 ms | 7255.85 ms | 182.70 ms |

All 30 recorded addition runs and all 30 multiplication runs were correct for every width (240 recorded results total).
