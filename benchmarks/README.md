# Benchmark measurements

[BENCHMARKS.md](../BENCHMARKS.md) preserves the September 5, 2026 measurements
on Apple M4 and Ryzen 9 7950X3D. These are historical results for their recorded
source and toolchains, not current-checkout performance claims. The raw records,
dependency locks, source hashes and machine conditions are retained unchanged.

Later campaigns:

- [f32 optimizations, September 5](2026-09-05/f32-optimization/README.md)
- [f64 optimizations, September 5](2026-09-05/f64-optimization/README.md)
- [binary128 trig, September 9](2026-09-09/trig-v2/README.md)
- [Intel atan2f and cbrt, September 11](2026-09-11/intel-issue9/README.md)

## Measurement policy

Use measured same-run metallic / CORE-MATH ratios for performance claims.
[Static analysis](../ANALYSIS.md) explains instruction dependencies, path costs,
fallback coverage and table footprints. Its representative input is not a
workload average: input bands and independent versus dependent calls can reverse
rankings. Coverage percentages must use the timed workload's distribution before
being used to weight an average cost.

- During optimization, benchmark the affected functions before and after on one
  primary x86-64 GNU/Linux host. Run benchmarks sequentially, record load and
  memory pressure, and repeat small or unstable differences. Same-run ratios do
  not eliminate contention bias.
- Check relevant portable f32/f64 changes on the existing Apple M4 configuration;
  refresh broader results periodically and before releases with performance claims.
  Use x86-64 GNU/Linux for binary128 with the current CORE-MATH dependencies.
- Measure on Intel hardware when making Intel-specific claims or tuning for it.
  A full processor/ISA matrix is unnecessary. ISA flags on a modern CPU do not
  emulate older hardware; test distinct build configurations when they change
  generated code, including baseline versus FMA-enabled builds.
- Preserve workload benchmarks; add narrow-band or dependent-call measurements
  when the optimization question needs them. Label the input distribution and
  whether RNG is timed.
- Keep noisy shared-runner timings out of pass/fail CI gates. Static checks and
  correctness tests remain suitable for CI.

Use `tools/benchmark_snapshot.py --help` for a new complete snapshot. Select
matching Rust and CORE-MATH CPU flags (`RUSTFLAGS` and `TARGET_CPU`), a fresh
target/output directory, and preserve the recorded dependency lock. Do not
combine estimates left by unrelated runs. New campaigns get new dated directories;
old records are not overwritten. `tools/benchmark_report.py` renders the specific
September 5 three-host archive and audits its 112-function coverage; update its
explicit assumptions before using it for a different campaign.
