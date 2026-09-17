# Metallic

[![Crates.io](https://img.shields.io/crates/v/metallic.svg)](https://crates.io/crates/metallic)
[![Documentation](https://docs.rs/metallic/badge.svg)](https://docs.rs/metallic)
[![Build status](https://github.com/jdh8/metallic-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jdh8/metallic-rs)

A fast correctly rounded math library in Rust!

See [BENCHMARKS.md](BENCHMARKS.md) for archived measurements on Apple M4 and
AMD Ryzen 9 7950X3D, with same-run CORE-MATH ratios and recorded source/toolchain
provenance. [ANALYSIS.md](ANALYSIS.md) complements them with static path costs,
accurate-leg coverage and table footprints; measured workloads support performance
claims. The [measurement policy](benchmarks/README.md) describes refresh cadence
and hardware coverage.

This library is a successor to [Metallic], my C library for WebAssembly
started in 2017.  Its most wanted feature turned out to be math functions I
wrote from scratch, so I decided to rewrite them in Rust.

[Metallic]: https://github.com/jdh8/metallic

## Development story

In 2021, I published my methods for implementing math functions in
[how to program math functions](https://jdh8.org/how-to-program-math-functions/).
In 2026, I turned those methods into agent skills and started vibe
optimizations.  Since April, nearly all new code has been AI-generated
under my direction.  The methods guiding that work are available as the
[Rust agent skill](https://github.com/jdh8/metallic-rs/blob/main/.claude/skills/program-math-functions/SKILL.md)
and the [C agent skill](https://github.com/jdh8/metallic/blob/main/.claude/skills/program-math-functions/SKILL.md).

Thanks to Paul Zimmermann and the other [CORE-MATH] contributors for their
pioneering work.  CORE-MATH demonstrates that a fast correctly rounded math
library is feasible.  CORE-MATH acts as both a correctness oracle and a
performance baseline.  Its algorithms are a valuable structural reference.

## Usage

The functions follow the C / libm naming convention: bare names operate on
`f64`, and the `f` suffix marks the `f32` variant.  Like [CORE-MATH], Metallic
focuses on difficult correctly rounded functions rather than the complete
libc/libm surface.  Every C99 transcendental ships in both precisions, plus
selected C23 functions and extensions: `sinpi`, `cospi`, `tanpi`, `asinpi`,
`acospi`, `atanpi`, `atan2pi`, `exp2m1`, `exp10m1`, `log2p1`, `log10p1`,
`rsqrt`, and `compound` (and their `f` variants) — all correctly rounded.

[CORE-MATH]: https://core-math.gitlabpages.inria.fr/

```rust
assert_eq!(metallic::exp(0.0), 1.0); // f64
assert_eq!(metallic::expf(0.0_f32), 1.0); // f32
```

Binary128 functions use the libquadmath-compatible `q` suffix and are gated
behind a nightly-only feature:

```toml
[dependencies]
metallic = { version = "0.3.0", features = ["f128"] }
```

```rust,ignore
#![feature(f128)]

assert_eq!(metallic::sqrtq(4.0_f128), 2.0);
assert_eq!(metallic::powq(2.0_f128, 0.5), core::f128::consts::SQRT_2);
```

The binary128 functions include `erfq`, `erfcq`, `tgammaq`, `lgammaq`,
`asinhq`, `acoshq`, `atanhq`, `sinhq`, `coshq`, `tanhq`, `sinpiq`,
`cospiq`, `tanpiq`, `asinpiq`, `acospiq`, `atanpiq`, `atan2piq`, `exp2m1q`,
and `exp10m1q`. See
[BINARY128.md](BINARY128.md) for the full list, how each one works, and how
metallic compares with glibc and libquadmath.

## Enable [fused multiply-add][fma] for best performance

This crate leans heavily on the fused multiply-add instruction.  Most modern
targets &mdash; `AArch64`, RISC-V with `Zfa`, WebAssembly with `relaxed-simd`
&mdash; enable it by default.  On x86-64, however, Rust's default `generic`
target does not, so the crate silently falls back to a slower path with
[double rounding][double-rounding] and emits a `cargo:warning` from `build.rs`
at compile time.

To opt in, ask `rustc` for a CPU baseline that includes FMA.  Pick whichever is
most convenient:

- **Per project** &mdash; commit a `.cargo/config.toml`:

  ```toml
  [target.'cfg(target_arch = "x86_64")']
  rustflags = ["-Ctarget-cpu=x86-64-v3"]   # AVX2 + FMA, portable across modern CPUs
  ```

- **Per user** &mdash; put the same snippet in `~/.cargo/config.toml`; it
  applies to every crate you build.
- **Per invocation** &mdash; `RUSTFLAGS="-Ctarget-cpu=x86-64-v3" cargo build --release`.

For maximum local performance, use `-Ctarget-cpu=native` instead so the
compiler is free to use every instruction your CPU supports (e.g. AVX-512).
Avoid `native` for redistributed binaries or shared CI caches: a binary built
on a newer CPU will trap with `SIGILL` on an older one.

[fma]: https://en.wikipedia.org/wiki/Multiply%E2%80%93accumulate_operation
[double-rounding]: https://en.wikipedia.org/wiki/Rounding#Double_rounding

## Assumptions

C libraries tend to have strict yet obsolete assumptions on math functions.
For example, `float` functions dare not use `double` instructions for fear
that the host does not support them.  In this library, I assume all Rust
primitive types are IEEE 754 compliant and native to the host.  In other
words, I assume the following instructions are available to floating-point
types:

- Addition, subtraction, multiplication, division
- Square root ([`f32::sqrt`](https://doc.rust-lang.org/std/primitive.f32.html#method.sqrt))
- Fused multiply-add ([`f32::mul_add`](https://doc.rust-lang.org/std/primitive.f32.html#method.mul_add))
- Rounding instructions such as [`f32::trunc`](https://doc.rust-lang.org/std/primitive.f32.html#method.trunc)

The assumptions beyond the four basic arithmetic operations creates dependency
on the [Rust standard library](https://doc.rust-lang.org/std/).

Besides, I ignore the floating-point environment, which is not available in
Rust.  It is also mostly unused in C and C++ because it requires `#pragma STDC
FENV_ACCESS ON` and compiler support.  Therefore, the only rounding mode in this
library is the default [rounding half to even][round-even].

[round-even]: https://en.wikipedia.org/wiki/Rounding#Rounding_half_to_even

## Goals

- The functions should be correctly rounded (error ≤ 0.5 ulp).
  - Works in progress may be only faithfully rounded (error < 1 ulp).  These
    functions are considered buggy until I make them correctly rounded.
- The functions should be about as fast as the system library.
- Try to make `f32` functions faster than the system library.

### Non-goals

- Most simple rounding and utility functions, such as `rint`, `trunc`, and
  `fabs`, are out of scope.  They typically map to a single instruction or an
  existing Rust primitive method; Metallic focuses on functions whose correct
  rounding requires nontrivial algorithms.

## Milestones

- [x] C99 transcendental and special functions for `f32` and `f64`, all
      correctly rounded:
  - Trigonometric: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`
  - Hyperbolic: `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`
  - Exponential, logarithmic, power, and root: `exp`, `exp2`, `expm1`, `log`,
    `log2`, `log10`, `log1p`, `pow`, `cbrt`, `hypot`
  - Error and gamma: `erf`, `erfc`, `tgamma`, `lgamma`
- [x] Selected C23 functions and extensions for `f32` and `f64`, all correctly
      rounded: `sinpi`, `cospi`, `tanpi`, `asinpi`, `acospi`, `atanpi`,
      `atan2pi`, `exp10`, `exp2m1`, `exp10m1`, `log2p1`, `log10p1`, `rsqrt`,
      `compound`, and `sincos`
- [ ] Complex `f32` and `f64` functions from [`<complex.h>`][complex]
- [ ] Expand the real `f128`/binary128 functions (`q` suffix; nightly) — see
      [BINARY128.md](BINARY128.md)

[complex]: https://en.cppreference.com/w/c/numeric/complex
