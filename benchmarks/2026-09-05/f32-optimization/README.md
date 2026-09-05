# f32 optimization measurements

Six functions optimized from `ced8edf`, measured on 2026-09-05. The arithmetic is
portable Rust. Only the atan2 quadrant selection differs on x86: opaque integer
masks prevent LLVM from reintroducing unpredictable branches. Other targets use
ordinary conditional selects; AArch64 lowers them to `fsub`/`fcsel`.

## Results

Criterion median ns per iteration, including fresh random-input generation.
M/CORE means metallic / CORE-MATH from the same run; lower is better.
Before and after each use their own paired CORE-MATH baseline.

| Function | M4 ns, before → after | M4 M/CORE | Ryzen ns, before → after | Ryzen M/CORE |
| --- | ---: | ---: | ---: | ---: |
| `atan2f` | 29.05 → 13.06 | 2.10 → 0.94× | 74.83 → 24.37 | 3.14 → 0.99× |
| `atan2pif` | 57.88 → 13.31 | 4.31 → 0.96× | 78.58 → 24.80 | 3.34 → 1.02× |
| `erff` | 29.53 → 8.30 | 2.81 → 0.78× | 34.62 → 17.46 | 2.27 → 1.16× |
| `erfcf` | 24.44 → 8.03 | 2.84 → 0.95× | 43.10 → 17.75 | 2.62 → 1.08× |
| `log2p1f` | 13.85 → 12.00 | 1.02 → 0.89× | 19.35 → 19.65 | 1.02 → 1.03× |
| `log10p1f` | 14.14 → 11.33 | 0.76 → 0.61× | 20.53 → 18.78 | 0.84 → 0.76× |

[Raw median estimates and confidence intervals](results.json) retain every pair.
All six selected implementations measured below CORE-MATH on M4, although the
near-parity rows need caution. On Ryzen, atan2 is at parity, log10p1f is faster,
erf/erfc retain smaller gaps, and log2p1f is effectively unchanged (its ~1–2%
relative movement is below the stated 3% noise floor).

The two atan2 benches use independent value-uniform inputs in `[-1,1]`;
atan2pif's earlier representation-uniform range was replaced for this campaign,
and **both** its before and after measurements use the new active band. The M4
before run linked that updated benchmark against the saved, unchanged native
library. Erf uses `[-6,6]`, erfc uses `[-6,10]`, and the two logarithms use the
existing representation-uniform `-1.0..` domain. Other benchmark ranges remain
unchanged. See the benchmark source files for exact samplers.

Rust 1.98.1; CORE-MATH/core-math-sys 1.3.0; fast_polynomial 0.4.0; Criterion 0.8.2.
Rust target flags: `-Ctarget-cpu=native` on Apple M4 and
`-Ctarget-cpu=x86-64-v3` on Ryzen 9 7950X3D. CORE-MATH's `TARGET_CPU` was native
and x86-64-v3 respectively. Each target ran serially on its host, with 0.5 s
warm-up, 2 s measurement, 60 samples, and 10000 bootstrap resamples. Ryzen runs
were pinned to CPU 15; its existing IDL batch job consumed roughly 2950% CPU,
with about 45 GiB available RAM and no swap. Mac measurements had normal UI
activity and approximately 75% free memory by `memory_pressure`. These workloads
can affect timing, especially close ratios; these are host-specific results.

## Changes and validation

- `atan2f` / `atan2pif`: independently generated degree-15 Chebyshev polynomial,
  one f64 division, shared relative rounding gate, original double-double
  fallback. The polynomial has a documented analytic interpolation bound.
  Actual x86 assembly has one divide and no quadrant branches. Making the
  coefficient table opaque saved instructions on ARM but did not improve time,
  so that experiment was discarded. Opaque quadrant masks improved Ryzen
  atan2pif from about 28.5 to 24.5 ns; ordinary selects were ~1 ns faster on M4.
- `erff` / `erfcf`: 155 direct degree-10 erfc cells replace the general path's
  division and exponential; the table occupies 13,640 bytes. Original accurate
  fallback retained. Clamping erf magnitude to 4 removes its saturation branch:
  `erfc(4) < 2^-25`, so every larger magnitude rounds to 1 by monotonicity.
  `tools/gen_erf_f32.py` generates all 1,705 coefficients independently and
  audits an analytic interpolation plus coefficient/evaluation error bound
  below 0.193 of the table gate. No fitted coefficients were imported.
- `log2p1f` / `log10p1f`: unchanged approximation arms evaluated together and
  selected by integer masks, eliminating the unpredictable band branch.
  Signed zeros, nonfinite values, and hard-tie fixes are preserved.

Validation commands:

```sh
cargo fmt --check
cargo test
cargo test --lib --features mpfr f32_:: -- --nocapture
# On x86-64 GNU/Linux:
TARGET_CPU=x86-64-v3 RUSTFLAGS=-Ctarget-cpu=x86-64-v3 \
  cargo test --release --test all f32_::
TARGET_CPU=x86-64-v3 RUSTFLAGS=-Ctarget-cpu=x86-64-v3 \
  cargo test --release --lib --features mpfr f32_:: -- --nocapture
cargo test --test all f32_::atan2
```

The default suite checks every one of the 2^32 binary32 representations for each
changed univariate function. Atan2 checks retain the hard-case corpora and add a
4-million-point grid, 4 million full-range pairs, and signed axis/diagonal/
subnormal/exceptional cases; atan2pif also retains its own 9-million-point grid
and 4-million-pair wide scan. The x86 f32 suite passed all 56 tests; its final
quadrant-mask change was additionally checked with the 6 atan2 tests.

MPFR soundness checks passed on both hosts. Worst measured `|error| / gate`:

| Leg | Worst ratio | Margin |
| --- | ---: | ---: |
| atan2f | 0.103668 | 9.6× |
| atan2pif | 0.103686 | 9.6× |
| erf | 0.124941 | 8.0× |
| erfc, positive | 0.054610 | 18.3× |
| erfc, negative | 0.249996 | 4.0× |

## Suggested commit message

```text
perf(f32): accelerate atan2, erf, and log1p variants

Add original polynomial fast legs with certified rounding gates and retained
accurate fallbacks; remove unpredictable logarithm and erf saturation branches.
Use opaque quadrant masks on x86 to prevent LLVM branch reconstruction.

M4 median ns and same-run CORE-MATH ratios, before -> after:
atan2f 29.05 -> 13.06 (2.10 -> 0.94x)
atan2pif 57.88 -> 13.31 (4.31 -> 0.96x, active band in both runs)
erff 29.53 -> 8.30 (2.81 -> 0.78x)
erfcf 24.44 -> 8.03 (2.84 -> 0.95x)
log2p1f 13.85 -> 12.00 (1.02 -> 0.89x)
log10p1f 14.14 -> 11.33 (0.76 -> 0.61x)

MPFR worst error/gate: atan2 0.103686 (9.6x margin), erf 0.124941
(8.0x), erfc positive 0.054610 (18.3x), negative 0.249996 (4.0x).
Validation: cargo fmt, cargo test, exhaustive f32 gates, new MPFR
certifications on AArch64 and x86-64-v3, x86 default atan2 corpus/grid checks.
```
