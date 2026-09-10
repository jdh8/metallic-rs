# Binary128 trig review, 2026-09-09

Rust baseline: `2c1facb`, with the corrected six-/ten-limb reduction windows,
six sine Taylor coefficients, seven cosine terms, and gate 64. The optimized
version uses the independent degree-four sine fit, six cosine terms and gate 16.

| Function | Corrected baseline (ns) | Optimized (ns) | Reduction |
|---|---:|---:|---:|
| Rust `sinq` | 78.03 | 71.11 | 8.9% |
| Rust `cosq` | 77.57 | 71.99 | 7.2% |

Criterion medians, serial runs, `RUSTFLAGS=-Ctarget-cpu=x86-64-v3 CC=clang
cargo +nightly bench --features f128 --bench sinq --bench cosq`. The compiled
binaries were run with `--bench --save-baseline trig-fixed` / `trig-v2`.
`python3 tools/bench_ratio.py median` was also run: these functions remain
unpaired because the released CORE-MATH Rust crate has no sine/cosine binding.
The libquadmath lanes in `results.json` are faithful-only comparisons.

| C function | v1 (TSC ticks) | v2 (TSC ticks) | Forced accurate (TSC ticks) |
|---|---:|---:|---:|
| `sinq` | 448.679 | 390.429 | 13271.122 |
| `cosq` | 431.352 | 371.621 | 13370.875 |

C v1 is `88b43290`; v2 is rebased onto `e6c6cff0`. Commands:

```sh
CC=gcc CORE_MATH_PERF_MODE=rdtsc CORE_MATH_LAUNCHER="taskset -c 0"   PERF_COUNT=10000 PERF_REPEAT=50 ./perf.sh sinq
# Same for cosq; force the accurate path by making the fast call's condition false.
```

These are the minima of 20 trials on CPU 0, using the harness's default
`[-pi, pi]` input band. RDTSC timer ticks are not hardware core cycles and
cannot be compared directly with the reviewer's 256/7100-cycle figures.
The initial unpinned C runs varied heavily; the table uses the pinned rerun.
A background IDLE workload remained active; our validation jobs were paused
for timing, and available memory was about 40 GiB. Treat small differences
as noise. The observed C time reductions were 13.0% and 13.8%.

Validation:

- `cargo fmt` and `cargo test` pass; the final default run has 18 library,
  207 integration and one passing doctest (one existing ignored doctest).
- Nightly `nextest --features f128`: 345 passed.
- Nightly `nextest --features "f128 mpfr"`: 477 passed, one existing skipped test.
- Trig fast-frame worst error/gate: 0.2334 (4.28x margin); tangent: 0.1184 (8.44x).
- Both coefficient generators check the Q128 fit on a dense mpmath grid
  (maximum about 2^-115.0208), then certify error < 2^-114 by exact rational
  arithmetic: Taylor through u^5, its u^6/15! remainder, and a derivative
  bound between 8193 grid points on [0, 3.765e-5]. The resulting bound is
  approximately 2^-114.2335. The accurate-leg coefficients remain Taylor.
- Expanded Rust corpora: 95,321 sine, 96,265 cosine, 95,383 tangent cases.
  They expose 344 / 726 / 1,057 misroundings with the original Rust windows;
  every case passes after widening. Each includes 6,906 new fast-window inputs.
- C worst-case checks: 98,017 sine and 98,961 cosine inputs, all four modes pass.
- C special checks: all four modes for each function pass with the default
  100-million sample setting (three loops of 33,333,333 draws per mode).
- Nightly Clippy fails with 16 `suboptimal_flops` errors in unchanged f32/f64
  code. The original `d0a3372` checkout produces exactly the same diagnostics;
  see `validation.txt`. No numerical implementation outside trig was changed.

The full MPFR run also exposed a pre-existing test-sampler error in `powq`:
rounding the base to 64 bits erased offsets near one and overflowed exponent
selection in debug builds. A separate test-only commit preserves 113 bits
and checks both neighbours of one; its four focused checks and the full suite pass.
