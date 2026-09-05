# Benchmarks

Measured 2026-09-05 (Asia/Taipei; UTC timestamps below) on the local machine and `dl02.skymizer.com`. The tables cover all **112 exposed math functions**: 46 f32 and 46 f64 functions on each host, plus all 20 opt-in binary128 functions on dl02. Each default run also includes nine gamma band measurements, reported separately.

Later [f64 optimization measurements](benchmarks/2026-09-05/f64-optimization/README.md) record portable fast-path improvements on both hosts.

Subsequent improvements to six f32 functions are recorded in the [f32 optimization measurements](benchmarks/2026-09-05/f32-optimization/README.md), with paired before/after results and correctness checks.

Times are Criterion **median nanoseconds per iteration**. **M/CORE = metallic / CORE-MATH** from the same benchmark run; below 1.00× favors metallic. Compare implementations within each row. Times characterize the recorded hosts, toolchains, and input distributions; they imply no universal ranking. A dash means that the harness has no corresponding comparison lane.

## Method and provenance

[The runner](tools/benchmark_snapshot.py) builds with `cargo bench --locked --no-run`, then executes one benchmark target at a time. Each lane uses 1.0 seconds of warm-up, 2.0 seconds of requested measurement time, 100 samples, and 10000 bootstrap resamples. The archived median estimates and confidence intervals are preserved in each `results.json`; displayed ratios use the unrounded point estimates. Sampling noise and changing machine load still apply to close ratios.

The timed iteration includes **fresh random-input generation and the function call**; the baseline lanes independently draw from the same distributions. RNG overhead can dominate inexpensive primitives such as FMA. These are sampled workloads, rather than isolated instruction latencies or exhaustive correctness tests.

[Shared f32/f64 samplers](benches/bench.rs) use value-uniform draws for bounded ranges, representation-uniform draws for open-ended ranges, and random significands with uniformly selected exponents for `Exponents` / `PositiveExponents`. `Exponents` also randomizes the sign. Full-representation draws include subnormals, infinities, and NaNs. [Binary128 samplers](benches/bench128.rs) assemble the floating bits directly; [log1pq](benches/log1pq.rs) has its own sign-dependent exponent band. Every function name below links to its exact input ranges and lanes. Functions and precisions use different bands: some retain domain errors or early returns, while others concentrate on the active kernel. The powq base uses positive magnitudes so random negative noninteger powers do not dominate its timing.

CORE-MATH is the principal comparison because it shares metallic's correct-rounding target. Standard-library, Rust `libm`, and GCC libquadmath lanes provide additional context; transcendental accuracy contracts differ. On dl02, most `f128::` methods call glibc; the `sqrtq` std lane resolves to Rust `compiler_builtins`, and the `exp10q` system lane is explicitly `glibc::exp10f128`. See [README baseline details](README.md#baselines). There is no CORE-MATH lane for `log2q`, `log10q`, `log1pq`, `powq`, `sinq`, `cosq`, or `tanq`; their libquadmath times remain separate from the CORE-MATH ratio. `compound` has no equivalent comparison entry point, so its row contains metallic alone.

The local AArch64 default target enables hardware FMA. On dl02, both Rust's `RUSTFLAGS` and CORE-MATH's `TARGET_CPU` select `x86-64-v3`, which includes FMA.

The measured source is based on commit `77e271f7ce393d246450de194f1598e59063c008` with six added benchmark targets (`atan2f`, `erff`, `erfcf`, `fma`, `fmaf`, `compound`) and the powq positive-base sampler correction. All three snapshots have identical library/harness file hashes and dependency locks. The source manifest and working-tree state are archived in the results. Locked benchmark dependencies: `core-math 1.3.0`, `core-math-sys 1.3.0`, `criterion 0.8.2`, `libm 0.2.16`, `rand 0.10.1`. Canonical benchmark-source SHA-256 (Cargo manifests/lock, build script, source, benchmark files, and any project Cargo config): `d85c75efaac6bf67e45fe77274cbf310fb9121a77fbf51f87fe66ceea400bbf4`. Remote raw manifests additionally record macOS AppleDouble `._*` resource-fork sidecars introduced during transfer. They are not compiled and are excluded from the canonical source comparison; the archived full manifests remain unchanged.

## Hosts and run conditions

| Recorded setting | Local | dl02 f32/f64 | dl02 binary128 |
| --- | --- | --- | --- |
| Host | MacBook-Pro-3.local | skymizer-DL02 | skymizer-DL02 |
| CPU | Apple M4 | AMD Ryzen 9 7950X3D 16-Core Processor | AMD Ryzen 9 7950X3D 16-Core Processor |
| Logical CPUs | 10 | 32 | 32 |
| RAM | 16.00 GiB | 61.98 GiB | 61.98 GiB |
| OS / architecture | macOS-26.6.2-arm64-arm-64bit | Linux-6.8.0-138-generic-x86_64-with-glibc2.35 | Linux-6.8.0-138-generic-x86_64-with-glibc2.35 |
| Rust / LLVM | rustc 1.98.1 (48a229cea 2026-09-01); LLVM 22.1.8 | rustc 1.98.1 (48a229cea 2026-09-01); LLVM 22.1.8 | rustc 1.100.0-nightly (a69a63265 2026-09-03); LLVM 23.1.1 |
| Cargo | cargo 1.98.1 (797e8a9bc 2026-08-05) | cargo 1.98.1 (797e8a9bc 2026-08-05) | cargo 1.100.0-nightly (b2e9d5f9d 2026-09-02) |
| C compiler | Apple clang version 21.0.0 (clang-2100.1.1.101) | Ubuntu clang version 14.0.0-1ubuntu1.1 | Ubuntu clang version 14.0.0-1ubuntu1.1 |
| RUSTFLAGS | empty | -Ctarget-cpu=x86-64-v3 | -Ctarget-cpu=x86-64-v3 |
| CFLAGS | unset | unset | unset |
| TARGET_CPU | native | x86-64-v3 | x86-64-v3 |
| Power / frequency policy | Now drawing from 'AC Power' | energy_performance_preference=balance_performance; scaling_driver=amd-pstate-epp; scaling_governor=powersave; scaling_max_freq=5759000; scaling_min_freq=400000 | energy_performance_preference=balance_performance; scaling_driver=amd-pstate-epp; scaling_governor=powersave; scaling_max_freq=5759000; scaling_min_freq=400000 |
| CPU affinity | OS scheduler | 15 | 15 |
| Measurements started (UTC) | 2026-09-04T22:28:47+00:00 | 2026-09-04T22:28:09+00:00 | 2026-09-04T22:46:55+00:00 |
| Run completed (UTC) | 2026-09-04T22:47:40+00:00 | 2026-09-04T22:46:49+00:00 | 2026-09-04T22:50:26+00:00 |
| Sampled machine state | 1-minute load 2.52–16.12 | 1-minute load 30.76–37.69; available RAM 44.25–45.43 GiB; maximum swap used 0.00 GiB | 1-minute load 32.88–33.91; available RAM 44.40–45.41 GiB; maximum swap used 0.00 GiB |
| Full metadata and estimates | [results.json](benchmarks/2026-09-05/local/results.json) | [results.json](benchmarks/2026-09-05/dl02/results.json) | [results.json](benchmarks/2026-09-05/dl02-f128/results.json) |

The local Mac had background UI activity in its process samples (`WindowServer`, `Google Chrome Helper`, `Google Chrome Helper (Renderer)`). On dl02, the concurrent `dump-teacher` batch in the `IDL` scheduling class persisted in the process samples, reporting up to 2903.0% CPU ([recorded process conditions](benchmarks/2026-09-05/dl02/conditions.txt)). These background workloads can influence timing and within-run ratios; interpret small differences cautiously. Full machine-state records also retain memory/swap observations and the largest CPU and memory consumers. The load ranges above cover observations during measurement.

**Local binary128: unavailable with the current benchmark dependencies.** The local Darwin C compiler rejects CORE-MATH's required `__float128` type ([recorded compiler probe](benchmarks/2026-09-05/local/f128-probe.txt)). The benchmark runner therefore requires x86-64 GNU/Linux for these targets; dl02 supplies all 20 measurements. This limitation concerns the benchmark dependency build, not the availability of metallic's opt-in Rust library code.

## Local — f32

| Function | metallic ns | CORE-MATH ns | M/CORE | std/system ns | libm ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| [acosf](benches/acosf.rs) | 9.28 | 9.64 | 0.96× | 13.38 | 12.92 |
| [acoshf](benches/acoshf.rs) | 8.80 | 9.04 | 0.97× | 8.89 | 10.02 |
| [acospif](benches/acospif.rs) | 6.54 | 6.61 | 0.99× | — | — |
| [asinf](benches/asinf.rs) | 9.34 | 9.56 | 0.98× | 11.70 | 11.65 |
| [asinhf](benches/asinhf.rs) | 7.53 | 8.30 | 0.91× | 9.14 | 9.06 |
| [asinpif](benches/asinpif.rs) | 6.43 | 7.01 | 0.92× | — | — |
| [atan2f](benches/atan2f.rs) | 28.70 | 13.75 | 2.09× | 19.93 | 28.04 |
| [atan2pif](benches/atan2pif.rs) | 47.69 | 67.54 | 0.71× | — | — |
| [atanf](benches/atanf.rs) | 9.14 | 8.99 | 1.02× | 9.05 | 8.55 |
| [atanhf](benches/atanhf.rs) | 7.34 | 7.81 | 0.94× | 7.32 | 15.04 |
| [atanpif](benches/atanpif.rs) | 9.03 | 9.04 | 1.00× | — | — |
| [cbrtf](benches/cbrtf.rs) | 5.05 | 6.15 | 0.82× | 5.51 | 4.97 |
| [compoundf](benches/compoundf.rs) | 15.47 | 18.18 | 0.85× | — | — |
| [cosf](benches/cosf.rs) | 11.07 | 9.97 | 1.11× | 9.58 | 32.20 |
| [coshf](benches/coshf.rs) | 7.30 | 6.54 | 1.12× | 7.40 | 8.55 |
| [cospif](benches/cospif.rs) | 6.58 | 6.13 | 1.07× | — | — |
| [erfcf](benches/erfcf.rs) | 24.21 | 8.49 | 2.85× | — | 16.09 |
| [erff](benches/erff.rs) | 28.36 | 10.36 | 2.74× | — | 16.22 |
| [exp10f](benches/exp10f.rs) | 7.46 | 8.34 | 0.89× | — | 10.62 |
| [exp10m1f](benches/exp10m1f.rs) | 6.25 | 7.10 | 0.88× | — | — |
| [exp2f](benches/exp2f.rs) | 6.71 | 7.38 | 0.91× | 6.04 | 7.44 |
| [exp2m1f](benches/exp2m1f.rs) | 6.19 | 6.39 | 0.97× | — | — |
| [expf](benches/expf.rs) | 6.89 | 7.22 | 0.95× | 6.07 | 8.41 |
| [expm1f](benches/expm1f.rs) | 6.60 | 6.33 | 1.04× | 10.07 | 14.15 |
| [fmaf](benches/fmaf.rs) | 15.07 | — | — | — | 15.18 |
| [frexpf](benches/frexpf.rs) | 4.96 | — | — | — | 5.02 |
| [hypotf](benches/hypotf.rs) | 10.01 | 9.92 | 1.01× | 9.82 | 10.33 |
| [ldexpf](benches/ldexpf.rs) | 9.64 | — | — | — | 12.26 |
| [lgammaf](benches/lgammaf.rs) | 13.72 | 15.28 | 0.90× | — | 16.45 |
| [log10f](benches/log10f.rs) | 10.61 | 10.94 | 0.97× | 10.88 | 11.62 |
| [log10p1f](benches/log10p1f.rs) | 14.23 | 19.27 | 0.74× | — | — |
| [log1pf](benches/log1pf.rs) | 12.14 | 14.16 | 0.86× | 13.65 | 13.81 |
| [log2f](benches/log2f.rs) | 10.28 | 11.06 | 0.93× | 11.18 | 11.49 |
| [log2p1f](benches/log2p1f.rs) | 13.74 | 13.56 | 1.01× | — | — |
| [logf](benches/logf.rs) | 11.03 | 10.77 | 1.02× | 11.16 | 10.97 |
| [powf](benches/powf.rs) | 22.93 | 22.93 | 1.00× | 15.19 | 23.68 |
| [roundf](benches/roundf.rs) | 4.82 | — | — | 4.74 | 8.06 |
| [rsqrtf](benches/rsqrtf.rs) | 9.89 | 10.32 | 0.96× | — | — |
| [sincosf](benches/sincosf.rs) | 11.58 | 10.73 | 1.08× | 12.50 | 32.01 |
| [sinf](benches/sinf.rs) | 10.95 | 10.38 | 1.06× | 8.71 | 31.93 |
| [sinhf](benches/sinhf.rs) | 7.15 | 6.68 | 1.07× | 7.63 | 13.67 |
| [sinpif](benches/sinpif.rs) | 6.52 | 6.33 | 1.03× | — | — |
| [tanf](benches/tanf.rs) | 10.71 | 10.11 | 1.06× | 10.94 | 28.77 |
| [tanhf](benches/tanhf.rs) | 7.78 | 8.23 | 0.95× | 9.09 | 9.49 |
| [tanpif](benches/tanpif.rs) | 6.18 | 6.46 | 0.96× | — | — |
| [tgammaf](benches/tgammaf.rs) | 16.34 | 21.85 | 0.75× | — | 48.14 |

## Local — f64

| Function | metallic ns | CORE-MATH ns | M/CORE | std/system ns | libm ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| [acos](benches/acos.rs) | 14.26 | 13.88 | 1.03× | 16.44 | 15.76 |
| [acosh](benches/acosh.rs) | 12.89 | 17.00 | 0.76× | 12.99 | 15.16 |
| [acospi](benches/acospi.rs) | 14.68 | 16.05 | 0.91× | — | — |
| [asin](benches/asin.rs) | 14.58 | 13.86 | 1.05× | 14.46 | 14.38 |
| [asinh](benches/asinh.rs) | 18.99 | 16.71 | 1.14× | 17.68 | 19.50 |
| [asinpi](benches/asinpi.rs) | 15.14 | 16.15 | 0.94× | — | — |
| [atan](benches/atan.rs) | 22.40 | 21.10 | 1.06× | 23.61 | 20.38 |
| [atan2](benches/atan2.rs) | 40.49 | 33.43 | 1.21× | 36.83 | 40.17 |
| [atan2pi](benches/atan2pi.rs) | 40.64 | 40.62 | 1.00× | — | — |
| [atanh](benches/atanh.rs) | 12.69 | 12.89 | 0.98× | 11.68 | 17.80 |
| [atanpi](benches/atanpi.rs) | 23.69 | 22.09 | 1.07× | — | — |
| [cbrt](benches/cbrt.rs) | 8.87 | 10.33 | 0.86× | 6.40 | 8.82 |
| [compound](benches/compound.rs) | 24.48 | — | — | — | — |
| [cos](benches/cos.rs) | 24.02 | 25.17 | 0.95× | 19.99 | 23.31 |
| [cosh](benches/cosh.rs) | 9.39 | 10.24 | 0.92× | 8.78 | 10.90 |
| [cospi](benches/cospi.rs) | 11.01 | 10.58 | 1.04× | — | — |
| [erf](benches/erf.rs) | 12.42 | 11.08 | 1.12× | — | 20.32 |
| [erfc](benches/erfc.rs) | 20.79 | 21.81 | 0.95× | — | 20.19 |
| [exp](benches/exp.rs) | 9.17 | 9.19 | 1.00× | 13.33 | 10.20 |
| [exp10](benches/exp10.rs) | 9.27 | 9.55 | 0.97× | — | 29.69 |
| [exp10m1](benches/exp10m1.rs) | 9.97 | 14.30 | 0.70× | — | — |
| [exp2](benches/exp2.rs) | 9.35 | 9.20 | 1.02× | 8.47 | 8.86 |
| [exp2m1](benches/exp2m1.rs) | 9.66 | 13.90 | 0.70× | — | — |
| [expm1](benches/expm1.rs) | 8.92 | 10.35 | 0.86× | 9.72 | 12.18 |
| [fma](benches/fma.rs) | 20.05 | — | — | — | 20.03 |
| [frexp](benches/frexp.rs) | 6.28 | — | — | — | 6.20 |
| [hypot](benches/hypot.rs) | 28.50 | 27.12 | 1.05× | 24.14 | 25.86 |
| [ldexp](benches/ldexp.rs) | 12.72 | — | — | — | 13.11 |
| [lgamma](benches/lgamma.rs) | 15.55 | 14.70 | 1.06× | — | 14.18 |
| [log](benches/log.rs) | 15.37 | 15.20 | 1.01× | 14.69 | 15.10 |
| [log10](benches/log10.rs) | 13.98 | 14.84 | 0.94× | 14.17 | 15.41 |
| [log10p1](benches/log10p1.rs) | 17.87 | 16.24 | 1.10× | — | — |
| [log1p](benches/log1p.rs) | 16.64 | 17.53 | 0.95× | 18.36 | 14.05 |
| [log2](benches/log2.rs) | 15.51 | 14.42 | 1.08× | 14.12 | 14.81 |
| [log2p1](benches/log2p1.rs) | 17.82 | 16.19 | 1.10× | — | — |
| [pow](benches/pow.rs) | 25.10 | 27.58 | 0.91× | 23.91 | 37.19 |
| [round](benches/round.rs) | 5.89 | — | — | 5.93 | 10.70 |
| [rsqrt](benches/rsqrt.rs) | 11.33 | 11.56 | 0.98× | — | — |
| [sin](benches/sin.rs) | 23.85 | 49.13 | 0.49× | 20.09 | 23.24 |
| [sincos](benches/sincos.rs) | 23.37 | 22.74 | 1.03× | 15.78 | 54.83 |
| [sinh](benches/sinh.rs) | 9.86 | 10.77 | 0.92× | 9.18 | 13.77 |
| [sinpi](benches/sinpi.rs) | 11.55 | 10.84 | 1.07× | — | — |
| [tan](benches/tan.rs) | 29.27 | 29.13 | 1.00× | 18.71 | 23.74 |
| [tanh](benches/tanh.rs) | 18.11 | 19.83 | 0.91× | 18.29 | 21.32 |
| [tanpi](benches/tanpi.rs) | 11.03 | 10.96 | 1.01× | — | — |
| [tgamma](benches/tgamma.rs) | 23.56 | 24.70 | 0.95× | — | 46.84 |

## dl02.skymizer.com — f32

| Function | metallic ns | CORE-MATH ns | M/CORE | std/system ns | libm ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| [acosf](benches/acosf.rs) | 14.20 | 13.67 | 1.04× | 18.84 | 17.44 |
| [acoshf](benches/acoshf.rs) | 17.66 | 17.67 | 1.00× | 19.07 | 18.14 |
| [acospif](benches/acospif.rs) | 10.93 | 13.07 | 0.84× | — | — |
| [asinf](benches/asinf.rs) | 13.96 | 13.48 | 1.04× | 18.46 | 16.60 |
| [asinhf](benches/asinhf.rs) | 13.85 | 12.82 | 1.08× | 15.54 | 15.89 |
| [asinpif](benches/asinpif.rs) | 10.73 | 13.02 | 0.82× | — | — |
| [atan2f](benches/atan2f.rs) | 74.61 | 23.82 | 3.13× | 40.94 | 36.19 |
| [atan2pif](benches/atan2pif.rs) | 70.42 | 120.54 | 0.58× | — | — |
| [atanf](benches/atanf.rs) | 12.73 | 12.48 | 1.02× | 13.96 | 12.13 |
| [atanhf](benches/atanhf.rs) | 15.35 | 14.09 | 1.09× | 26.53 | 25.22 |
| [atanpif](benches/atanpif.rs) | 12.84 | 12.70 | 1.01× | — | — |
| [cbrtf](benches/cbrtf.rs) | 8.79 | 10.26 | 0.86× | 9.80 | 8.81 |
| [compoundf](benches/compoundf.rs) | 28.80 | 33.55 | 0.86× | — | — |
| [cosf](benches/cosf.rs) | 17.27 | 15.40 | 1.12× | 15.69 | 47.19 |
| [coshf](benches/coshf.rs) | 12.24 | 11.23 | 1.09× | 15.87 | 17.72 |
| [cospif](benches/cospif.rs) | 9.58 | 10.35 | 0.93× | — | — |
| [erfcf](benches/erfcf.rs) | 43.10 | 16.41 | 2.63× | — | 29.77 |
| [erff](benches/erff.rs) | 34.69 | 15.61 | 2.22× | — | 28.87 |
| [exp10f](benches/exp10f.rs) | 12.67 | 13.58 | 0.93× | — | 16.37 |
| [exp10m1f](benches/exp10m1f.rs) | 11.49 | 12.47 | 0.92× | — | — |
| [exp2f](benches/exp2f.rs) | 11.73 | 12.87 | 0.91× | 11.41 | 13.56 |
| [exp2m1f](benches/exp2m1f.rs) | 11.41 | 11.53 | 0.99× | — | — |
| [expf](benches/expf.rs) | 12.39 | 12.75 | 0.97× | 11.18 | 17.38 |
| [expm1f](benches/expm1f.rs) | 11.93 | 10.36 | 1.15× | 24.19 | 21.71 |
| [fmaf](benches/fmaf.rs) | 20.85 | — | — | — | 21.83 |
| [frexpf](benches/frexpf.rs) | 6.43 | — | — | — | 6.36 |
| [hypotf](benches/hypotf.rs) | 14.01 | 15.46 | 0.91× | 12.22 | 13.43 |
| [ldexpf](benches/ldexpf.rs) | 13.06 | — | — | — | 21.17 |
| [lgammaf](benches/lgammaf.rs) | 22.86 | 23.02 | 0.99× | — | 28.47 |
| [log10f](benches/log10f.rs) | 18.30 | 18.56 | 0.99× | 22.45 | 20.50 |
| [log10p1f](benches/log10p1f.rs) | 21.08 | 27.09 | 0.78× | — | — |
| [log1pf](benches/log1pf.rs) | 20.98 | 19.79 | 1.06× | 24.85 | 20.87 |
| [log2f](benches/log2f.rs) | 18.00 | 17.74 | 1.01× | 17.72 | 20.24 |
| [log2p1f](benches/log2p1f.rs) | 20.26 | 19.66 | 1.03× | — | — |
| [logf](benches/logf.rs) | 20.61 | 18.37 | 1.12× | 17.80 | 18.83 |
| [powf](benches/powf.rs) | 29.07 | 32.78 | 0.89× | 28.95 | 37.96 |
| [roundf](benches/roundf.rs) | 5.43 | — | — | 5.15 | 11.34 |
| [rsqrtf](benches/rsqrtf.rs) | 16.65 | 17.20 | 0.97× | — | — |
| [sincosf](benches/sincosf.rs) | 18.49 | 17.49 | 1.06× | 15.88 | 49.12 |
| [sinf](benches/sinf.rs) | 17.15 | 16.37 | 1.05× | 15.98 | 48.93 |
| [sinhf](benches/sinhf.rs) | 13.69 | 5.92 | 2.31× | 23.99 | 24.09 |
| [sinpif](benches/sinpif.rs) | 12.36 | 12.56 | 0.98× | — | — |
| [tanf](benches/tanf.rs) | 16.26 | 16.09 | 1.01× | 19.75 | 47.45 |
| [tanhf](benches/tanhf.rs) | 11.36 | 12.55 | 0.91× | 16.99 | 16.96 |
| [tanpif](benches/tanpif.rs) | 9.61 | 10.18 | 0.94× | — | — |
| [tgammaf](benches/tgammaf.rs) | 27.81 | 29.99 | 0.93× | — | 102.43 |

## dl02.skymizer.com — f64

| Function | metallic ns | CORE-MATH ns | M/CORE | std/system ns | libm ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| [acos](benches/acos.rs) | 21.57 | 22.32 | 0.97× | 25.33 | 19.68 |
| [acosh](benches/acosh.rs) | 26.82 | 30.51 | 0.88× | 26.46 | 29.53 |
| [acospi](benches/acospi.rs) | 22.44 | 23.80 | 0.94× | — | — |
| [asin](benches/asin.rs) | 22.43 | 22.58 | 0.99× | 25.03 | 19.96 |
| [asinh](benches/asinh.rs) | 42.44 | 40.42 | 1.05× | 40.00 | 45.74 |
| [asinpi](benches/asinpi.rs) | 23.25 | 37.08 | 0.63× | — | — |
| [atan](benches/atan.rs) | 36.54 | 34.59 | 1.06× | 35.28 | 33.35 |
| [atan2](benches/atan2.rs) | 69.26 | 68.75 | 1.01× | 65.52 | 61.39 |
| [atan2pi](benches/atan2pi.rs) | 71.75 | 74.20 | 0.97× | — | — |
| [atanh](benches/atanh.rs) | 23.63 | 21.87 | 1.08× | 29.14 | 27.51 |
| [atanpi](benches/atanpi.rs) | 39.12 | 38.77 | 1.01× | — | — |
| [cbrt](benches/cbrt.rs) | 18.85 | 20.91 | 0.90× | 23.86 | 23.79 |
| [compound](benches/compound.rs) | 49.51 | — | — | — | — |
| [cos](benches/cos.rs) | 48.11 | 44.64 | 1.08× | 36.71 | 36.85 |
| [cosh](benches/cosh.rs) | 15.80 | 16.98 | 0.93× | 18.13 | 22.56 |
| [cospi](benches/cospi.rs) | 21.71 | 22.44 | 0.97× | — | — |
| [erf](benches/erf.rs) | 23.52 | 22.14 | 1.06× | — | 33.82 |
| [erfc](benches/erfc.rs) | 40.92 | 45.03 | 0.91× | — | 35.79 |
| [exp](benches/exp.rs) | 16.21 | 15.41 | 1.05× | 19.45 | 21.12 |
| [exp10](benches/exp10.rs) | 16.33 | 15.87 | 1.03× | — | 62.41 |
| [exp10m1](benches/exp10m1.rs) | 17.07 | 21.76 | 0.78× | — | — |
| [exp2](benches/exp2.rs) | 15.12 | 14.13 | 1.07× | 18.25 | 12.69 |
| [exp2m1](benches/exp2m1.rs) | 17.32 | 22.11 | 0.78× | — | — |
| [expm1](benches/expm1.rs) | 16.80 | 17.26 | 0.97× | 23.61 | 24.01 |
| [fma](benches/fma.rs) | 26.23 | — | — | — | 27.58 |
| [frexp](benches/frexp.rs) | 8.16 | — | — | — | 8.12 |
| [hypot](benches/hypot.rs) | 58.99 | 61.03 | 0.97× | 59.48 | 56.45 |
| [ldexp](benches/ldexp.rs) | 21.97 | — | — | — | 22.30 |
| [lgamma](benches/lgamma.rs) | 29.22 | 28.78 | 1.02× | — | 25.19 |
| [log](benches/log.rs) | 25.41 | 24.65 | 1.03× | 22.28 | 23.46 |
| [log10](benches/log10.rs) | 25.63 | 25.18 | 1.02× | 26.41 | 24.66 |
| [log10p1](benches/log10p1.rs) | 34.60 | 33.93 | 1.02× | — | — |
| [log1p](benches/log1p.rs) | 32.30 | 29.95 | 1.08× | 31.19 | 32.12 |
| [log2](benches/log2.rs) | 25.31 | 23.91 | 1.06× | 23.68 | 24.20 |
| [log2p1](benches/log2p1.rs) | 34.40 | 35.57 | 0.97× | — | — |
| [pow](benches/pow.rs) | 30.75 | 29.89 | 1.03× | 42.20 | 85.71 |
| [round](benches/round.rs) | 7.31 | — | — | 6.85 | 13.33 |
| [rsqrt](benches/rsqrt.rs) | 20.11 | 22.24 | 0.90× | — | — |
| [sin](benches/sin.rs) | 50.06 | 87.59 | 0.57× | 37.93 | 37.05 |
| [sincos](benches/sincos.rs) | 38.87 | 32.29 | 1.20× | 43.63 | 77.94 |
| [sinh](benches/sinh.rs) | 18.44 | 18.57 | 0.99× | 24.95 | 28.11 |
| [sinpi](benches/sinpi.rs) | 22.33 | 23.48 | 0.95× | — | — |
| [tan](benches/tan.rs) | 59.91 | 51.76 | 1.16× | 42.54 | 42.25 |
| [tanh](benches/tanh.rs) | 38.92 | 41.02 | 0.95× | 49.46 | 50.51 |
| [tanpi](benches/tanpi.rs) | 21.88 | 23.64 | 0.93× | — | — |
| [tgamma](benches/tgamma.rs) | 46.69 | 51.98 | 0.90× | — | 102.62 |

## dl02.skymizer.com — binary128

| Function | metallic ns | CORE-MATH ns | M/CORE | std/system ns | libquadmath ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| [acosq](benches/acosq.rs) | 78.44 | 82.85 | 0.95× | 930.77 | — |
| [asinq](benches/asinq.rs) | 71.82 | 71.11 | 1.01× | 888.44 | — |
| [atan2q](benches/atan2q.rs) | 99.15 | 97.30 | 1.02× | 701.94 | — |
| [atanq](benches/atanq.rs) | 68.27 | 68.63 | 0.99× | 607.92 | — |
| [cbrtq](benches/cbrtq.rs) | 52.86 | 55.05 | 0.96× | 674.29 | — |
| [cosq](benches/cosq.rs) | 77.32 | — | — | 652.54 | 647.20 |
| [exp10q](benches/exp10q.rs) | 52.21 | 52.32 | 1.00× | 1646.56 | — |
| [exp2q](benches/exp2q.rs) | 52.63 | 50.71 | 1.04× | 865.83 | — |
| [expm1q](benches/expm1q.rs) | 52.24 | 55.48 | 0.94× | 995.73 | — |
| [expq](benches/expq.rs) | 51.99 | 51.56 | 1.01× | 806.46 | — |
| [hypotq](benches/hypotq.rs) | 74.36 | 75.01 | 0.99× | 880.40 | — |
| [log10q](benches/log10q.rs) | 54.18 | — | — | 877.94 | 839.09 |
| [log1pq](benches/log1pq.rs) | 59.54 | — | — | 1221.48 | 1161.62 |
| [log2q](benches/log2q.rs) | 50.02 | — | — | 782.07 | 754.86 |
| [logq](benches/logq.rs) | 51.91 | 50.15 | 1.04× | 859.52 | — |
| [powq](benches/powq.rs) | 109.20 | — | — | 2292.66 | 2224.89 |
| [rsqrtq](benches/rsqrtq.rs) | 35.06 | 36.49 | 0.96× | — | — |
| [sinq](benches/sinq.rs) | 76.04 | — | — | 647.76 | 644.34 |
| [sqrtq](benches/sqrtq.rs) | 39.45 | 37.37 | 1.06× | 65.41 | — |
| [tanq](benches/tanq.rs) | 91.92 | — | — | 790.97 | 782.62 |

## Supplementary gamma bands

These nine workloads per host reuse three public functions over narrower input bands. They are additional workloads and are excluded from the 112-function coverage count.

### Local

| Function | metallic ns | CORE-MATH ns | M/CORE | std/system ns | libm ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| [lgamma_recur](benches/lgamma.rs) | 13.43 | 13.23 | 1.02× | — | 12.33 |
| [lgamma_reflect](benches/lgamma.rs) | 27.06 | 26.40 | 1.02× | — | 32.11 |
| [lgamma_stirling](benches/lgamma.rs) | 12.34 | 13.12 | 0.94× | — | 12.31 |
| [lgammaf_rational](benches/lgammaf.rs) | 7.05 | 13.00 | 0.54× | — | — |
| [lgammaf_reflect](benches/lgammaf.rs) | 15.73 | 14.56 | 1.08× | — | — |
| [lgammaf_stirling](benches/lgammaf.rs) | 9.37 | 9.81 | 0.96× | — | — |
| [tgamma_recur](benches/tgamma.rs) | 17.94 | 22.85 | 0.79× | — | 42.32 |
| [tgamma_reflect](benches/tgamma.rs) | 24.66 | 26.61 | 0.93× | — | 51.73 |
| [tgamma_stirling](benches/tgamma.rs) | 19.56 | 21.08 | 0.93× | — | 45.53 |

### dl02.skymizer.com

| Function | metallic ns | CORE-MATH ns | M/CORE | std/system ns | libm ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| [lgamma_recur](benches/lgamma.rs) | 27.08 | 26.69 | 1.01× | — | 22.46 |
| [lgamma_reflect](benches/lgamma.rs) | 56.40 | 59.66 | 0.95× | — | 59.25 |
| [lgamma_stirling](benches/lgamma.rs) | 24.96 | 26.62 | 0.94× | — | 22.46 |
| [lgammaf_rational](benches/lgammaf.rs) | 14.02 | 19.15 | 0.73× | — | — |
| [lgammaf_reflect](benches/lgammaf.rs) | 26.68 | 25.03 | 1.07× | — | — |
| [lgammaf_stirling](benches/lgammaf.rs) | 17.93 | 16.67 | 1.08× | — | — |
| [tgamma_recur](benches/tgamma.rs) | 38.08 | 48.86 | 0.78× | — | 82.51 |
| [tgamma_reflect](benches/tgamma.rs) | 51.71 | 58.14 | 0.89× | — | 105.69 |
| [tgamma_stirling](benches/tgamma.rs) | 42.16 | 48.92 | 0.86× | — | 106.26 |

## Reproducing the snapshots

Use the recorded Rust and C compiler versions, the same measured source, and the archived lock file. The commands below use fresh output and target directories: the runner refuses existing measurements so older Criterion data cannot enter a new snapshot. Run the two dl02 commands serially on `dl02.skymizer.com`; run the local command on the local host. The f128 runner selects `+nightly --features f128` and needs `CC=clang`. Preserve that nightly version when reproducing these measurements.

### local

```sh
cp benchmarks/2026-09-05/local/Cargo.lock.snapshot Cargo.lock
env CC=cc RUSTFLAGS= TARGET_CPU=native python3 tools/benchmark_snapshot.py --precision default --target-dir target/benchmark-repeat-local --output benchmarks/repeat/local --warm-up-time 1.0 --measurement-time 2.0 --sample-size 100 --nresamples 10000
```

### dl02

```sh
cp benchmarks/2026-09-05/dl02/Cargo.lock.snapshot Cargo.lock
env CC=clang RUSTFLAGS=-Ctarget-cpu=x86-64-v3 TARGET_CPU=x86-64-v3 python3 tools/benchmark_snapshot.py --precision default --target-dir target/benchmark-repeat-dl02 --output benchmarks/repeat/dl02 --warm-up-time 1.0 --measurement-time 2.0 --sample-size 100 --nresamples 10000 --cpu 15
```

### dl02-f128

```sh
cp benchmarks/2026-09-05/dl02-f128/Cargo.lock.snapshot Cargo.lock
env CC=clang RUSTFLAGS=-Ctarget-cpu=x86-64-v3 TARGET_CPU=x86-64-v3 python3 tools/benchmark_snapshot.py --precision f128 --target-dir target/benchmark-repeat-dl02-f128 --output benchmarks/repeat/dl02-f128 --warm-up-time 1.0 --measurement-time 2.0 --sample-size 100 --nresamples 10000 --cpu 15
```

Regenerate this report from the committed snapshots:

```sh
python3 tools/benchmark_report.py --snapshots benchmarks/2026-09-05
```

[The report generator](tools/benchmark_report.py) validates 92/92/20 public-function coverage, all nine supplementary bands in each default run, archived lock hashes, matching source files, and consumption of every recorded lane. Raw Criterion files and build/run logs remain in the target directories recorded in each snapshot.
