# metallic

[![Crates.io](https://img.shields.io/crates/v/metallic.svg)](https://crates.io/crates/metallic)
[![Documentation](https://docs.rs/metallic/badge.svg)](https://docs.rs/metallic)
[![Build status](https://github.com/jdh8/metallic-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jdh8/metallic-rs)
[![Benchmark status](https://github.com/jdh8/metallic-rs/actions/workflows/bench.yml/badge.svg)](https://jdh8.github.io/metallic-rs/dev/bench/)

A fast correctly rounded math library in Rust!

This library is a successor to [Metallic], my C library for WebAssembly
started in 2017.  Its most wanted feature turned out to be math functions I
wrote from scratch, so I decided to rewrite them in Rust.

[Metallic]: https://github.com/jdh8/metallic

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
assert_eq!(metallic::rsqrtq(4.0_f128), 0.5);
assert_eq!(metallic::cbrtq(8.0_f128), 2.0);
assert_eq!(metallic::exp2q(10.0_f128), 1024.0);
assert_eq!(metallic::logq(1.0_f128), 0.0);
assert_eq!(metallic::log2q(1024.0_f128), 10.0);
```

Run its tests with `cargo +nightly test --features f128`. `sqrtq`, `rsqrtq`
and `cbrtq` seed a fixed-point z<sup>&minus;1/2</sup> / z<sup>&minus;1/3</sup>
from degree-2 Taylor tables (function values, no fitted coefficients) and keep
the midpoint walk only for the rounding-tie window their guard bits cannot
decide; `sqrtq` rides the same seed as `rsqrtq` and corrects s = r·z by the
same power series in r²z &minus; 1.
`expq`, `exp2q` and `exp10q` share one fixed-point engine for 2<sup>x·L</sup>,
with a 256-bit accurate leg behind the Ziv gate of the 128-bit fast one.
`expm1q` reuses that engine above 2<sup>&minus;6</sup>; below, a Taylor
correction rides on top of the exact input significand, so subtracting 1 from
2<sup>f</sup> &isin; [1, 2) never gets to cancel the bits that matter.
`logq` reduces in log space instead: an 18-bit estimate of
log<sub>2</sub>(m) picks three 31-bit reciprocals whose product with the
significand is exact, so the logarithms to add back are the only table the sum
needs; `log2q` is the same engine with base-2 tables, exact at every power of
two.  `atan2q` reduces on dyadic breakpoints: one float divide picks
`i ≈ round(64·min/max)`, the 6-bit dyadic `i/64` makes both sides of
tan(θ &minus; atan(i/64)) exact 128-bit integers, and one hardware 128-by-64
divide seeds the Newton reciprocal that divides them to 128 or 384 bits before
the atan(i/64) table and the quadrant offset add back &mdash; every variable
shift along the way cut out of 64-bit limbs, since LLVM has no 128-bit funnel
shift.

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
      [Binary128 status](#binary128-status)

### Binary128 status

Each function is done when both gates hold:

- **CR** — correctly rounded, all strict gates green (bit-exact vs
  `core_math::<fn>q` on the worst-case corpus, deterministic samples, MPFR).
  `sinq`/`cosq`/`tanq`/`log2q` have no CORE-MATH binding yet: their strict
  gate replays a home-grown corpus that carries its MPFR answers, and
  CORE-MATH's `f64` `sin`/`cos`/`tan`/`log2` cross-check them oracle-free.
- **Perf** — same-run median ratio `metallic::<fn>q / core_math::<fn>q` ≈ 1×
  or better (`RUSTFLAGS=-Ctarget-cpu=x86-64-v3 cargo +nightly bench
  --features f128 --bench <fn>q`, then `python3 tools/bench_ratio.py median`).

| Function | CR | Perf (ratio vs CORE-MATH) |
|----------|:--:|:--|
| `acosq`  | ✅ | 0.97× |
| `asinq`  | ✅ | 1.03× |
| `atanq`  | ✅ | 1.02× |
| `atan2q` | ✅ | 1.01× |
| `cbrtq`  | ✅ | 0.89× |
| `cosq`   | ✅ | n/a — CORE-MATH has no `cosq`; 8× faster than libquadmath's faithful `cosq` |
| `exp10q` | ✅ | 0.93× |
| `exp2q`  | ✅ | 0.94× |
| `expm1q` | ✅ | 0.94× |
| `expq`   | ✅ | 0.96× |
| `hypotq` | ✅ | 1.01× |
| `log2q`  | ✅ | n/a — CORE-MATH has no `log2q`; 15× faster than libquadmath's faithful `log2q` |
| `logq`   | ✅ | 1.00× |
| `rsqrtq` | ✅ | 0.76× |
| `sinq`   | ✅ | n/a — CORE-MATH has no `sinq`; 8× faster than libquadmath's faithful `sinq` |
| `sqrtq`  | ✅ | 0.87× |
| `tanq`   | ✅ | n/a — CORE-MATH has no `tanq`; 8.5× faster than libquadmath's faithful `tanq` |

`atanq` rides `atan2q` — `atan2(x, 1)` is exactly `atan(x)` — but folds the
unit operand through its own reduction: the sector is an integer shift
(`|x| < 1`) or one hardware divide (`|x| ≥ 1`) instead of the float divide,
and below the first breakpoint the reduced tangent *is* the input significand,
so both legs skip the quotient and its Newton reciprocal outright.  The shared
legs, tables, and Ziv gate carry over, and the folded path ships its own
soundness certification.

`asinq` and `acosq` split at `|x| = 2^-3`.  Below it the arc sine is its own
reduced argument: no root is formed at all and the fast leg is a bare Taylor
series, in floating form for `asin` and summed into `π/2 ∓ ·` for `acos`.  The
band charges each binade for its own width — fourteen terms below 2^-4,
nineteen in the top binade alone — because the root pipeline costs 2.7× the
series there and the extra terms must not be billed to the binades that never
needed them.  Above the band both are the `atan2q` pipeline fed a wide
`√(1−x²)`:
`1 − x²` is exact in fixed point, the root reaches 2^-207 (fast leg) and
2^-305 (accurate leg), and the reduction runs in 256/384-bit limbs before
rejoining `atan2q`'s legs, tables, and certified Ziv gate.

`sinq` and `cosq` reduce by Payne–Hanek on 64-bit limbs of 2/π: the window
starts at the limb the exponent points to, everything above the units bit but
its two low bits is a multiple of 4 and drops, and the product with the
significand leaves the quadrant and a fraction — 192 bits on the fast leg,
448 on the accurate one.  The fraction rounds to `n = round(256·x/π)`, a
quadrant and a breakpoint `j·π/256`, and the residual `|g| ≤ 1/256`
normalizes into a floating fraction at its own exponent, so `θ = g·π/2`
keeps full relative precision however close `x` sits to a multiple of π/2.
Six Taylor terms in `θ²` (eighteen on the accurate leg) and a 128-entry
`sin`/`cos(j·π/256)` table recombine in `atan2q`'s frames; below 2^-8 the
argument is its own reduced angle, below 2^-57 the results are `x` and 1.

`tanq` shares that reduction and evaluates `tan θ` by one Taylor chain (eight
terms, twenty-five on the accurate leg; the coefficients are exact Bernoulli
rationals), then recombines by the addition formula `tan(j·π/256 + θ) =
(T_j + tan θ)/(1 − T_j·tan θ)` from a table of `tan(j·π/256)`: numerator and
denominator never cancel, and an odd quadrant just swaps them (`−cot`).  The
quotient is `atan2q`'s hardware-seeded Newton reciprocal of the denominator's
top limb plus one Newton step on the quotient itself against the full
denominator (two at 384 bits on the accurate leg), every iterate held below
the ratio so no residual goes negative.

[complex]: https://en.cppreference.com/w/c/numeric/complex
