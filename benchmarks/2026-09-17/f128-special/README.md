# Binary128 error and gamma functions — 2026-09-17

Four new original implementations, measured on an Intel Core i9-14900K
with four logical CPUs online and `-Ctarget-cpu=x86-64-v3`. These are
same-run Criterion median nanoseconds per iteration, including the input
sampler. CORE-MATH has no binary128 binding for these functions. libquadmath
and nightly std have weaker accuracy contracts and serve as timing context.

| Function | Metallic ns | libquadmath ns | M/quadmath | std ns | Signed input binades |
| --- | ---: | ---: | ---: | ---: | --- |
| `erfcq` | 71.96 | 725.37 | 0.099× | 698.52 | -8..=6 |
| `erfq` | 54.78 | 567.06 | 0.097× | 542.17 | -20..=3 |
| `lgammaq` | 1813.07 | 1851.53 | 0.979× | 1608.43 | -8..=10 |
| `tgammaq` | 2177.45 | 2839.07 | 0.767× | 2541.61 | -8..=10 |

Each lane used a 1-second warm-up, 2-second measurement, 50 samples and
1,000 bootstrap resamples. Targets ran serially. The exhaustive tests, MPFR
certifications and corpus generators were paused during all measurements
and resumed afterward. Background editor processes remained. Close differences
are subject to noise; this does not establish a universal hardware ranking.

[results.json](results.json) preserves the full estimates, confidence
intervals, compiler/hardware metadata, dependency-lock hash and source-file
hashes. [timing.log](timing.log) records the run and paused process IDs.
[Cargo.lock.snapshot](Cargo.lock.snapshot) records the dependencies.
The source hashes identify the measured working tree; later formatting and
additional test-only certification cases do not change the production kernels.

Reproduce a target with:

```sh
CC=clang RUSTFLAGS=-Ctarget-cpu=x86-64-v3 cargo +nightly bench \
  --features f128 --bench tgammaq -- \
  --warm-up-time 1 --measurement-time 2 --sample-size 50 --nresamples 1000
```

The same command accepts `erfq`, `erfcq`, or `lgammaq`. `std` names are
`f128::erf`, `f128::erfc`, `f128::gamma`, and `f128::ln_gamma` respectively.

The separate [MPFR accuracy survey](accuracy-survey.log) used 200,000 draws
per function. Metallic had no misroundings; maximum errors for libquadmath
and dynamically resolved glibc were 1.1718, 2.3357, 5.9367, and 6.0830 ulps
for erf, erfc, gamma, and log-Gamma. Reproduce with:

```sh
CC=clang cargo +nightly run --release --features 'f128 mpfr' \
  --example f128_ulp_survey -- 200000 erfq erfcq tgammaq lgammaq
```
