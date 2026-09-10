# f64 optimization measurements

Portable Rust optimizations from `7f4f07b`, measured on 2026-09-05. No architecture-specific implementation was needed. Correct-rounding fallbacks and special-case handling are retained.

## Results

Criterion median ns per iteration, including fresh random-input generation. Each M/CORE ratio divides metallic by CORE-MATH from the same process; lower is better. Before and after each have their own paired baseline.

| Function | M4 ns, before → after | M4 M/CORE | Ryzen ns, before → after | Ryzen M/CORE |
| --- | ---: | ---: | ---: | ---: |
| `atan2` | 38.94 → 33.74 | 1.20 → 1.01× | 70.75 → 62.75 | 1.00 → 0.91× |
| `atan2pi` | 39.45 → 34.01 | 1.00 → 0.84× | 72.34 → 64.22 | 0.97 → 0.85× |
| `tan` | 28.38 → 26.92 | 1.00 → 0.93× | 59.28 → 49.53 | 1.15 → 0.96× |
| `log1p` | 16.30 → 16.30 | 0.98 → 0.96× | 31.81 → 31.01 | 1.07 → 1.06× |
| `log2p1` | 17.51 → 16.75 | 1.10 → 1.04× | 33.82 → 31.85 | 0.94 → 0.91× |
| `log10p1` | 17.76 → 17.02 | 1.11 → 1.05× | 33.90 → 31.45 | 1.02 → 0.94× |
| `erf` | 12.36 → 12.07 | 1.12 → 1.10× | 23.71 → 22.04 | 1.06 → 0.98× |
| `erfc` | 20.69 → 20.76 | 0.96 → 0.96× | 41.22 → 40.79 | 0.91 → 0.90× |

`atan2pi` and `tan` beat CORE-MATH on both hosts; `atan2` reaches parity on M4 and beats it on Ryzen. The base-2/base-10 log1p variants improve by about 4–7% in absolute time. Erf improves clearly on Ryzen; its smaller M4 change is close to the noise floor. `log1p` and `erfc` have no substantial measured change. Some functions remain slower than CORE-MATH.

[Raw estimates, confidence intervals, source hashes, and machine conditions](results.json) preserve the reported pairs. Times are specific to these input distributions and hosts, not universal latency guarantees. Near-parity ratios and changes below 3% should be treated cautiously. Repeated Ryzen atan2 measurements ranged from 0.91× to 0.99× CORE-MATH; the table uses the final repeated pair (62.75 / 68.69 ns). An earlier pass of that same final executable measured 68.64 / 69.46 ns. Background contention can move ratios as well as absolute times.

## Implementation

- **atan2 / atan2pi:** sort finite magnitudes without a swap branch and combine quadrant offsets in one signed fold. Compensate the small ratio's division error so its fast gate no longer rejects nearly every direct-quadrant result. Evaluate the fine-cell cubic correction separately, with independently accounted reduction, reconstruction, and base-conversion errors. New analytic gate bounds and a shared MPFR certificate cover all octants, floating scales, table seams, and residual-underflow thresholds. Actual M4 assembly confirms branchless sorting/reconstruction. Existing coefficient tables are reused unchanged.
- **tan:** below `|x| = 1/32`, evaluate six exact Taylor coefficients on the exact input and gate the cubic correction directly. The correction-scaled gate avoids unnecessary accurate refinements near zero. Coefficients are generated independently by exact rational division of the sine/cosine Taylor series; the reproducible command is beside the array. The general reduction and accurate fallback are unchanged.
- **log1p / log2p1 / log10p1:** keep the exact input separate from its correction, use six Taylor terms for the small band, and halve that band's gate to `2^-49 · x²`. The shared logarithm engine and all other bands retain their previous coefficients and bounds. Certificates include concentrated sampling near the band's upper edge.
- **erf / erfc:** evaluate the exact linear table lead independently of the polynomial tail; fold the small residuals first and add the larger correction through one final FMA. The certificate measures the actual unnormalized-pair gate and the negative-erfc consumer.

The shared trig identity shortcut, Cody–Waite subtraction rewrite, sincos quadrant rewrite, and a log2 assembly experiment were discarded because their gains were inconsistent or absent. Benchmark ranges are unchanged.

## Measurement setup

Rust 1.98.1; CORE-MATH/core-math-sys 1.3.0; fast_polynomial 0.4.0; Criterion 0.8.2. M4 uses `RUSTFLAGS=-Ctarget-cpu=native` and `TARGET_CPU=native`. Ryzen 9 7950X3D on `dl02.skymizer.com` uses `CC=clang`, `RUSTFLAGS=-Ctarget-cpu=x86-64-v3`, and `TARGET_CPU=x86-64-v3`, pinned to CPU 15. Benchmarks run strictly serially on each host, with 0.5 s warm-up, 2 s measurement, 60 samples, and 10000 bootstrap resamples. Baseline and modified executables are preserved separately for comparison.

Inputs are the existing benchmark samplers: independent signed log-uniform arguments with exponents -20 through 20 for atan2/atan2pi; signed exponents -26 through 19 for tan; signed exponents -50 through -1 for all three log1p variants; value-uniform `[-6,6]` for erf and `[-6,28]` for erfc. The baseline M4 source was built independently from the recorded commit, with the same dependency lock and flags.

The Mac had ordinary desktop activity. Ryzen had two existing IDL batch jobs (`solve-gen`, `dump-teacher`) occupying roughly 30 cores in aggregate, load near 63, about 36 GiB available memory, and no swap. Neither benchmark host ran our tests concurrently with its benchmarks. The archived conditions include process and memory observations.

Example reproduction for each target (repeat with the baseline checkout):

```sh
# Apple Silicon
TARGET_CPU=native RUSTFLAGS=-Ctarget-cpu=native \
  cargo bench --locked --bench atan2 -- \
  '^(metallic|core_math)::' --warm-up-time 0.5 --measurement-time 2 \
  --sample-size 60 --nresamples 10000 --noplot

# x86-64 GNU/Linux
CC=clang TARGET_CPU=x86-64-v3 RUSTFLAGS=-Ctarget-cpu=x86-64-v3 \
  taskset -c 15 cargo bench --locked --bench atan2 -- \
  '^(metallic|core_math)::' --warm-up-time 0.5 --measurement-time 2 \
  --sample-size 60 --nresamples 10000 --noplot
```

## Correctness validation

`cargo fmt --check` and `cargo test` pass on the final source: 18 unit tests, 207 integration tests, and the active doctest. The existing feature-gated binary128 doctest remains ignored by the default build. All 36 selected output/corpus/MPFR checks pass on x86-64-v3, covering every affected API. The independent erfc scan includes four million inputs through the subnormal boundary.

```sh
cargo fmt --check
cargo test
cargo test --release --locked --features mpfr --lib -- \
  f64_::atan::ziv_soundness::atan2 f64_::trig::ziv_soundness::trig_fast \
  f64_::erf::ziv_soundness::erf_table f64_::log::ziv_soundness::log1p_small \
  f64_::log::ziv_soundness::log2p1_small f64_::log::ziv_soundness::log10p1_small \
  --nocapture
# On x86-64 GNU/Linux:
CC=clang TARGET_CPU=x86-64-v3 RUSTFLAGS=-Ctarget-cpu=x86-64-v3 \
  cargo test --release --locked --features mpfr --test all -- \
  f64_::atan2:: f64_::atan2pi:: f64_::tan:: f64_::erf:: f64_::erfc:: \
  f64_::log1p:: f64_::log2p1:: f64_::log10p1::
```

All changed fast legs pass their MPFR soundness checks on M4 and x86-64-v3. Worst measured error divided by the gate:

| Leg | Worst error/gate | Margin |
| --- | ---: | ---: |
| atan2 | 0.269751 | 3.71× |
| atan2pi | 0.351711 | 2.84× |
| tan | 0.318889 | 3.14× |
| erf table | 0.329478 | 3.04× |
| negative erfc table | 0.108870 | 9.19× |
| log1p small | 0.3488 | 2.87× |
| log2p1 small | 0.3498 | 2.86× |
| log10p1 small | 0.3537 | 2.83× |

The minimum measured margin exceeds the required 2×. Expanded atan2 certification exposed missing error contributions near table seams and during quadrant/base reconstruction; these are covered by the new analytic bounds rather than by weakening a correctness test. No fitted coefficients were imported from another library.

An optional strict `cargo clippy --lib -- -D warnings` run fails on existing diagnostics outside the four changed source files, primarily generated f32 coefficient formatting. It reports no diagnostics in the changed f64 modules.

## Suggested commit message

```text
perf(f64): accelerate atan2, tangent, erf, and log1p variants

Use compensated atan2 ratios and one quadrant fold, a correction-scaled
small-tangent leg, lean erf table assembly, and shorter small-log kernels.
Keep the implementations portable and retain accurate rounding fallbacks.

M4 median ns and same-run CORE-MATH ratios, before -> after:
atan2 38.94 -> 33.74 (1.20 -> 1.01x)
atan2pi 39.45 -> 34.01 (1.00 -> 0.84x)
tan 28.38 -> 26.92 (1.00 -> 0.93x)
log2p1 17.51 -> 16.75 (1.10 -> 1.04x)
log10p1 17.76 -> 17.02 (1.11 -> 1.05x)

Ryzen same-run ratios: atan2 0.91x, atan2pi 0.85x, tan 0.96x,
log2p1 0.91x, log10p1 0.94x, erf 0.98x.
MPFR worst error/gate: atan2 0.269751, atan2pi 0.351711,
tan 0.318889, erf 0.329478, negative erfc 0.108870,
log1p 0.3488, log2p1 0.3498, log10p1 0.3537 (minimum margin 2.83x).

Validation: cargo fmt, cargo test (18 unit + 207 integration tests),
MPFR soundness on M4 and x86-64-v3, 36 selected x86 output/corpus/MPFR checks.
```
