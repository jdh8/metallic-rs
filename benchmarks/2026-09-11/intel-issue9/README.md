# Intel atan2f and cbrt optimization

This campaign addresses three functions in [issue #9](https://github.com/jdh8/metallic-rs/issues/9).
The baseline is `0736813`, whose implementations match the issue's `0718f8f`.
The exp family and lgamma remain open performance work.

`atan2f` and `atan2pif` now sort their finite magnitudes with ordered selects,
avoiding the NaN checks emitted for `min`/`max`. One constant table and a signed
multiply-add combine both quadrant reflections. This removes the two opaque
masks and their stack round trips. The half-turn scale multiplies the quotient
while the original polynomial evaluates. Neither polynomial coefficients nor
rounding gates change.

`cbrt` adds 1074 to the normalized exponent before dividing it by three. The
biased exponent is nonnegative even for the smallest subnormal, so the compiler
can use unsigned arithmetic without signed quotient/remainder corrections.
Subtracting 358 from the quotient restores the original exponent exactly.
The numerical kernel is unchanged. Its MPFR certificate shares the production
exponent helper, and a new oracle test covers every exponent boundary, including
every subnormal leading-bit position, adjacent values, and both signs.

## Measurements

The compact [records](results.json) contain two interleaved A/B rounds, median
confidence intervals, executable and source hashes, and machine conditions.
[Cargo.lock.snapshot](Cargo.lock.snapshot) preserves the dependency versions.
Each reported ratio divides metallic by CORE-MATH from the same executable run.
Both rounds improve all three functions; the table below uses the second round.

| Function | Before ns | After ns | CORE-MATH ratio, before → after |
| --- | ---: | ---: | ---: |
| `atan2f` | 10.89 | 9.48 | 1.25× → 1.10× |
| `atan2pif` | 10.94 | 9.51 | 1.22× → 1.06× |
| `cbrt` | 9.23 | 8.47 | 1.16× → 1.08× |

Intel Core i9-14900K, CPU 14 (P-core with its SMT sibling offline), Rust 1.98.1,
GCC 15.2.0, core-math/core-math-sys 1.3.0, fast_polynomial 0.4.0, Criterion 0.8.2.
Rust uses `-Ctarget-cpu=x86-64-v3`; the C oracle uses core-math-sys's default
`-march=native`. These flags are identical between arms. The executable's ELF
compiler metadata confirms GCC; this campaign does not claim the issue's
recorded Clang toolchain. No tests or other benchmarks ran alongside these
measurements. Load, available memory, and memory pressure are recorded per run.

The existing samplers are unchanged: independent value-uniform pairs in
`[-1,1]²` for both atan2 functions and representation-uniform f64 arguments for
cbrt. Random-input generation is timed. Each lane uses 1 s warm-up, 2 s
measurement, 100 samples, and 100,000 bootstrap resamples. Only this Intel host
was measured; Apple M4 and Zen 4 performance have not been checked in this campaign.

Reproduce each arm from its source with the saved dependency lock:

```sh
TARGET_CPU=native CC=cc RUSTFLAGS=-Ctarget-cpu=x86-64-v3 \
  taskset -c 14 cargo bench --locked --bench atan2f --bench atan2pif --bench cbrt -- \
  '^(metallic|core_math)::' --warm-up-time 1 --measurement-time 2 \
  --sample-size 100 --nresamples 100000 --noplot
python3 tools/bench_ratio.py median
```

Disassembly of the timed atan2f kernel confirms `vminsd`/`vmaxsd` without
unordered-input checks, a read-only offset lookup, and no quadrant branch or
stack stores. A locally constructed offset array caused a stack copy; named
constant tables avoid it.

## Experiments not retained

The direct bit test for f32 rounding saved less than 2%. A second angular
reduction allowed ten polynomial coefficients but LLVM introduced a branch,
raising atan2f to about 14.34 ns. Parallel cross terms in the exp table product
did not improve throughput; precomputed relative table tails saved less than 2%.
A 32-cell Taylor seed for cbrt was slower than the existing polynomial after
the unsigned exponent change. These experiments are absent from the final code.

## Validation

`cargo fmt --check` and `cargo test` pass: 18 unit tests, 208 integration
tests, and the active doctest. The existing feature-gated binary128 doctest
remains ignored by the default build.
The local global Cargo configuration sets `target-cpu=native`, so the default
suite uses that configuration. Separate explicit `x86-64` and `x86-64-v3` builds
check both sides of the hardware-FMA boundary.

All 10 selected integration tests pass in both explicit builds: the two atan2
corpora, grids, quadrants and extreme ratios, and the cbrt corpus, representation
sweep, exponent seams, and independent two-million-input MPFR sweep.

The in-source MPFR soundness certificates pass under native, x86-64-v3, and
baseline x86-64 flags. All three builds report the same sampled maxima:

| Fast leg | Worst error / gate | Margin |
| --- | ---: | ---: |
| atan2f | 0.103668 | 9.6× |
| atan2pif | 0.103530 | 9.7× |
| cbrt | 0.0624 | 16.0× |

```sh
cargo fmt --check
cargo test
# Repeat with x86-64-v3; omit RUSTFLAGS to use the local native configuration.
RUSTFLAGS=-Ctarget-cpu=x86-64 cargo test --release --features mpfr --lib -- \
  f32_::atan::ziv_soundness::atan2_fast_legs_are_sound \
  f64_::misc::ziv_soundness::cbrt_fast_leg_is_sound --nocapture
RUSTFLAGS=-Ctarget-cpu=x86-64 cargo test --release --features mpfr --test all -- \
  f32_::atan2f:: f32_::atan2pif:: f64_::cbrt::
```

## Proposed commit message

```text
perf: reduce Intel atan2f quadrant and cbrt exponent overhead

Sort finite atan2f magnitudes without NaN checks and combine the quadrant
reflections in one constant-table multiply-add, shared with atan2pif.
Bias cbrt's exponent by 1074 before unsigned division by three, preserving
the numerical kernel and covering every exponent seam against CORE-MATH.

i9-14900K repeated median ns and same-run CORE-MATH ratios, before -> after:
atan2f   10.89 -> 9.48 (1.25 -> 1.10x)
atan2pif 10.94 -> 9.51 (1.22 -> 1.06x)
cbrt      9.23 -> 8.47 (1.16 -> 1.08x)

MPFR worst error/gate: atan2f 0.103668, atan2pif 0.103530, cbrt 0.0624
(minimum margin 9.6x), in native, x86-64-v3, and baseline x86-64 builds.
Validation: cargo fmt, cargo test (18 unit + 208 integration tests and the
active doctest), and all 10 selected oracle tests on both explicit ISA builds.
Record the benchmark conditions, source hashes, and dependency lock.

Refs #9; exp and lgamma performance work remains open.
```
