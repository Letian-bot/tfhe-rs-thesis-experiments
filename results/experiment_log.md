# TFHE-rs Reproduction — Experiment Log

Running log of environment setup and experiment results. Add a new dated entry each time you run something new; once there's enough data, fold the relevant bits into Chapter 3 / Chapter 4 of the thesis.

---

## Environment (fixed, unless you change machine/toolchain)

| Item | Value |
|---|---|
| Rust version | rustc 1.97.1 (2026-07-14) |
| Cargo version | cargo 1.97.1 (2026-06-30) |
| TFHE-rs version | tfhe v1.6.3 |
| Build mode | release (optimized) |
| CPU | Intel Core i7-10870H @ 2.20GHz (8 cores / 16 threads) |
| RAM | 16.0 GB |
| OS | Windows (64-bit) |
| Project path | `E:\Gla\study\semester3\dissertation\reproduce\tfhe_test` |

---

## Entry 1 — 2026-07-XX — Environment sanity check

**Goal:** confirm toolchain, key generation, encryption, and PBS bootstrap all work end to end.

**Parameters used (toy, not security-rated):**
- LWE dimension: 742
- GLWE dimension: 1
- Polynomial size: 2048
- PBS base log: 23
- PBS level: 1

**What was run:** official Zama `core_crypto` tutorial example — computes 2 × 3 two ways (cleartext linear multiplication, and via Programmable Bootstrapping).

**Result:**
- Cleartext multiplication: expected 6, got 6 ✅
- PBS multiplication: expected 6, got 6 ✅
- First release build time: ~1 min 21 s

**Notes:** environment confirmed functional. Next step: replace the `|x: u64| 2 * x` lookup table with a NAND truth table over a packed two-bit input, to reproduce a homomorphic NAND gate (thesis Section 3.2).

---

## Entry 2 — 2026-07-27 — Boolean gate reproduction (official high-level API)

**Goal:** reproduce TFHE-rs's core homomorphic operations (encryption, decryption, basic gates) using the officially maintained `tfhe::boolean` high-level API, as the ground-truth reference before attempting a lower-level reproduction.

**Approach:** used `tfhe::boolean::prelude::*` directly — `ClientKey`/`ServerKey` from `gen_keys()`, `client_key.encrypt`/`decrypt`, and `server_key.nand/and/or/xor`. Internally these call TFHE-rs's own gate-bootstrapping implementation (linear combination of the two input ciphertexts + a single PBS with a fixed test polynomial + key switch); see `tfhe-rs/tfhe/src/boolean/engine/mod.rs` in the official repo for the reference implementation.

**Cargo.toml dependency:**
```toml
tfhe = { version = "~1.6.2", features = ["boolean"] }
```

**What was run:** exhaustively tested all four input combinations `(false,false), (false,true), (true,false), (true,true)` against NAND, AND, OR, and XOR.

**Result:** all 16 cases (4 gates × 4 input combinations) correct.
- NAND: 4/4 ✅
- AND: 4/4 ✅
- OR: 4/4 ✅
- XOR: 4/4 ✅

Build time: 33.17s (release).

**Notes:** this used the official high-level API rather than manually reconstructed core_crypto calls — this matches the thesis's actual 3.2 wording ("reproduce the core operations... and verify their correctness"), not the stricter "from the fundamental level using lower-level API" phrasing in the Motivation section, which still needs to be reconciled/reworded once the overall scope is finalised. Next possible step: a `core_crypto`-level reproduction of the same gates (encode bits as ±1/8, linear combine, single fixed-test-polynomial PBS, key switch) to compare against this high-level result — not yet attempted.

---

## Entry 3 — 2026-07-27 — Parameter set comparison

**Goal:** compare TFHE-rs's official published boolean parameter sets on key generation time, NAND gate correctness, and average NAND gate evaluation time (thesis Section 3.4).

**Parameter sets used (from `tfhe::boolean::parameters`, tfhe v1.6.3 — note: exact dimensions differ from the historical GitHub source I originally checked, since Zama updates these values across releases as security estimates get refreshed; the numbers below are what this tfhe version actually reports):**

| Parameter set | LWE dim | GLWE dim | Poly size | Key-gen time | NAND correctness | Avg NAND gate time (20 trials) |
|---|---|---|---|---|---|---|
| `DEFAULT_PARAMETERS` (error ≤ 2⁻⁴⁰) | 805 | 3 | 512 | 385.55 ms | PASS (4/4) | 15.60 ms |
| `PARAMETERS_ERROR_PROB_2_POW_MINUS_165` (error ≤ 2⁻¹⁶⁵) | 837 | 2 | 1024 | 471.70 ms | PASS (4/4) | 21.43 ms |
| `TFHE_LIB_PARAMETERS` (~120-bit security) | 630 | 1 | 1024 | 198.55 ms | PASS (4/4) | 11.71 ms |

**What was run:** for each parameter set — generate client/server keys (timed), verify NAND against all 4 input combinations, then time 20 repeated NAND gate calls and average.

**Result:** all three parameter sets produced correct NAND results on every case. Speed ranking (fastest to slowest gate evaluation): TFHE_LIB_PARAMETERS (11.71ms) < DEFAULT_PARAMETERS (15.60ms) < PARAMETERS_ERROR_PROB_2_POW_MINUS_165 (21.43ms) — consistent with the expected trade-off: lower target failure probability / larger polynomial size costs more per gate. Key-gen time did not follow the same simple ordering (DEFAULT_PARAMETERS took longer to generate keys than the lower-error-probability set despite being faster per gate), which is worth double-checking with more trials before treating as a firm conclusion.

**Notes:** single run only (n=1 for key-gen timing, n=20 for gate timing) on the usual dev machine (see Environment table above). For the thesis, this should be repeated across multiple runs / a larger trial count to get variance, and ideally cross-referenced against each parameter set's documented security level (bit-security, not just named error probability) from the TFHE-rs docs or Lattice Estimator, to make the correctness/performance/security three-way trade-off explicit rather than just showing time numbers.

---

## Entry 4 — 2026-08-03 — Empirical failure rate (2000 trials/set)

**Goal:** get an empirical correctness estimate beyond the 4-case exhaustive check, for the thesis 3.3 TODO.

**Approach:** keys generated once per parameter set; then 2000 independent NAND trials with random inputs and fresh ciphertexts each time (same keys reused).

**Result:** 0 failures out of 2000 for all three parameter sets.

| Parameter set | Trials | Failures | Total time | Time/trial |
|---|---|---|---|---|
| DEFAULT_PARAMETERS | 2000 | 0 | 29.71s | 14.86ms |
| PARAMETERS_ERROR_PROB_2_POW_MINUS_165 | 2000 | 0 | 43.00s | 21.50ms |
| TFHE_LIB_PARAMETERS | 2000 | 0 | 24.98s | 12.49ms |

**Notes:** 2000 trials can't meaningfully test a claimed 2^-40 (or 2^-165) failure bound — that would need astronomically more trials. What it *does* support: via the "rule of three" approximation, 0/2000 gives ~95% confidence the true failure rate is below ~3/2000 ≈ 0.15%. That's a genuine (if loose) empirical result, consistent with every parameter set's documented bound, not a restatement of it. Written into thesis Section 3.3.

---

## Entry 5 — 2026-08-03 — Parameter comparison, 5 repeated runs

**Goal:** address the "single run" limitation from Entry 3 — get mean/stddev instead of one-off numbers, for the thesis 3.4 TODO.

**Approach:** for each parameter set, repeated the full benchmark (fresh key generation + 20 NAND calls) 5 times, computed mean and standard deviation.

**Result:**

| Parameter set | Key-gen mean | Key-gen stddev | NAND mean | NAND stddev |
|---|---|---|---|---|
| DEFAULT_PARAMETERS | 272.07ms | 39.66ms | 13.74ms | 0.15ms |
| PARAMETERS_ERROR_PROB_2_POW_MINUS_165 | 383.82ms | 43.85ms | 19.13ms | 0.88ms |
| TFHE_LIB_PARAMETERS | 189.74ms | 4.77ms | 11.16ms | 0.10ms |

All 5 runs correct (4/4 NAND cases) for every parameter set.

**Notes:** ordering from Entry 3 holds up (TFHE_LIB fastest, ERROR_PROB_165 slowest on both metrics) — the gaps between means are much larger than the standard deviations, so this is a real effect, not noise. One new observation: the single-run numbers from Entry 3 (385.55/471.70/198.55ms key-gen) were all a bit higher than these 5-run means (272.07/383.82/189.74ms) — possibly a first-run warmup cost. Written into thesis Section 3.4, including this warmup caveat.

---

## Entry 6 — 2026-08-17 — Depth experiment (chained NAND)

**Goal:** test Chapter 2's central claim directly — that per-gate bootstrapping removes the depth ceiling, allowing unbounded-depth circuits. Single-gate experiments never accumulate the noise that would break a leveled scheme.

**Approach:** chained `ct = NAND(ct, ct)` = NOT ct, repeated to controllable depth. Value flips each gate so correct output at depth d is predictable (start XOR (d mod 2)). No absorbing state; every step is a real bootstrap (unlike NOT, which is linear/cheap in TFHE-rs). DEFAULT_PARAMETERS.

**Result:** correct at every depth, per-gate time essentially constant.

| Depth | Correct | Total | Per-gate |
|---|---|---|---|
| 1 | yes | 17.91ms | 17.91ms |
| 10 | yes | 168.23ms | 16.82ms |
| 100 | yes | 1.54s | 15.36ms |
| 1,000 | yes | 15.63s | 15.63ms |
| 10,000 | yes | 149.21s | 14.92ms |

**Notes:** 10,000 chained gates all correct — well past where a leveled scheme would fail. Per-gate time flat across 4 orders of magnitude → total time linear in depth. This is the strongest single piece of evidence for Chapter 2's core argument. Written into thesis Section 3.4 (new section).

---

## Entry 7 — 2026-08-17 — Real workload (integer types)

**Goal:** confirm depth behaviour on realistic computation, and exercise Chapter 2's claim that PBS is not limited to boolean gates (shortint/integer built on it). Needed Cargo.toml `features = ["boolean", "shortint", "integer"]` (high-level API + prelude live behind these).

**Approach:** high-level FheUintN add/multiply/compare at increasing bit width; wider integer = deeper internal circuit.

**Result:** all correct, including 64-bit multiply of large operands.

| Type | Add | Multiply | Compare (gt) |
|---|---|---|---|
| FheUint8 | 108ms | 175ms | 65ms |
| FheUint16 | 180ms | 626ms | — |
| FheUint32 | 285ms | 2,305ms | — |
| FheUint64 | 471ms | 7,592ms | — |

**Notes:** time rises with width (deeper circuit). Multiply/add gap widens 1.6× (8-bit) → 16× (64-bit), consistent with O(n²) vs O(n). Comparison works too (non-arithmetic PBS use). 64-bit multiply ≈ 7.6s / ~15ms baseline ≈ ~500 bootstraps behind one `*`. Written into thesis Section 3.4.

---

<!--
When ready to write it up, this log maps onto:
- Environment table -> Section 3.1 (Experimental Methodology) / 4.1 (Experimental Setup)
- Each entry's parameters + result -> Section 3.3 (Correctness Verification) and/or 4.2 (Results)
- Trends across entries with different parameter sets -> Section 3.4 (Parameter Sets) and 4.3 (Discussion)
-->
