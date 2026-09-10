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
assert_eq!(metallic::rsqrtq(4.0_f128), 0.5);
assert_eq!(metallic::cbrtq(8.0_f128), 2.0);
assert_eq!(metallic::exp2q(10.0_f128), 1024.0);
assert_eq!(metallic::logq(1.0_f128), 0.0);
assert_eq!(metallic::log2q(1024.0_f128), 10.0);
assert_eq!(metallic::log10q(1000.0_f128), 3.0);
assert_eq!(metallic::log1pq(1.0_f128), core::f128::consts::LN_2);
assert_eq!(metallic::powq(2.0_f128, 0.5), core::f128::consts::SQRT_2);
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
needs; `log2q` and `log10q` are the same engine with base-2 and base-10
tables, exact at every power of two and every representable power of ten, and
`log1pq` is the natural one behind an exact 256-bit `1 + x` (its argument is
its own reduction below 2<sup>&minus;18</sup>).
`powq` is 2<sup>y·log<sub>2</sub>x</sup> on both engines: the logarithm's
frame times the exact significand of `y` feeds the exponential's, with the
Ziv gate widened by `|y|`; a 384-by-256-bit accurate leg decides what the fast
one refuses, an exact-case detector rounds the dyadic-rational powers
(`x^y = m^N·2^k`, the only midpoints there are) from their integer value, and a
table-free 640-bit tier settles the rest.
`atan2q` reduces on dyadic breakpoints: one float divide picks
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
  `sinq`/`cosq`/`tanq`/`log2q`/`log10q`/`log1pq`/`powq` have no CORE-MATH
  binding yet: their strict gate replays a home-grown corpus that carries its
  MPFR answers, and CORE-MATH's `f64` `sin`/`cos`/`tan`/`log2`/`log10`/
  `log1p`/`pow` cross-check them oracle-free.
- **Perf** — measured same-run median ratio `metallic::<fn>q / core_math::<fn>q`
  ≈ 1× or better on the recorded workload and hardware. Archived results are in
  [BENCHMARKS.md](BENCHMARKS.md); [ANALYSIS.md](ANALYSIS.md) helps diagnose costs.
  To time one function on your own machine run
  `RUSTFLAGS=-Ctarget-cpu=x86-64-v3 cargo +nightly bench --features f128
  --bench <fn>q`, then `python3 tools/bench_ratio.py median`.  Each bench also
  carries a `f128::` lane (nightly `std`) and, where CORE-MATH has no binding,
  a `quadmath::` one; both are faithful-only, so they are context rather than
  the headline — see [Baselines](#baselines).

| Function | CR |
|----------|:--:|
| `acosq`  | ✅ |
| `asinq`  | ✅ |
| `atanq`  | ✅ |
| `atan2q` | ✅ |
| `cbrtq`  | ✅ |
| `cosq`   | ✅ |
| `exp10q` | ✅ |
| `exp2q`  | ✅ |
| `expm1q` | ✅ |
| `expq`   | ✅ |
| `hypotq` | ✅ |
| `log2q`  | ✅ |
| `log10q` | ✅ |
| `log1pq` | ✅ |
| `logq`   | ✅ |
| `powq`   | ✅ |
| `rsqrtq` | ✅ |
| `sinq`   | ✅ |
| `sqrtq`  | ✅ |
| `tanq`   | ✅ |

`atanq` rides `atan2q` — `atan2(x, 1)` is exactly `atan(x)` — but folds the
unit operand through its own reduction: the sector is an integer shift
(`|x| < 1`) or one hardware divide (`|x| ≥ 1`) instead of the float divide,
and below the first breakpoint the reduced tangent *is* the input significand,
so both legs skip the quotient and its Newton reciprocal outright.  The shared
legs, tables, and Ziv gate carry over, and the folded path ships its own
soundness certification.

`asinq` and `acosq` split at `|x| = 2^-3`.  Below it the arc sine is its own
reduced argument: no root is formed at all and the fast leg is a minimax
polynomial, in floating form for `asin` and summed into `π/2 ∓ ·` for `acos`.
Independent Remez fits use eleven terms below 2^-4 and fourteen in the top
binade, with exact rational error bounds after coefficient rounding.
Above it the fast leg reduces on dyadic *sine* breakpoints and never divides:
with `u = min(x, √(1−x²))` and
`v = max(·)`, the breakpoint `sin φ_j = j/128` is read off `u`'s top bits, and
`sin(asin(u) − φ_j) = u·cos φ_j − v·(j/128)` makes one side an exact
small-integer product and the other a 256-bit multiply by the tabulated
`cos φ_j`; a seven-term Taylor series finishes the difference, and
`asin(j/128)` adds back in `atan2q`'s frame.  `1 − x²` is exact in fixed point and its root is
`sqrtq`'s own integer frame plus one Newton step against all 256 bits.  The
accurate leg is the `atan2q` pipeline fed a 384-bit root, with its own
reduction, tables, and certified Ziv gate.

`sinq` and `cosq` reduce by Payne–Hanek on 64-bit limbs of 2/π: the window
starts at the limb the exponent points to, everything above the units bit but
its two low bits is a multiple of 4 and drops, and the product with the
significand leaves the quadrant and a fraction — 192 bits on the fast leg,
448 on the accurate one.  The fraction rounds to `n = round(256·x/π)`, a
quadrant and a breakpoint `j·π/256`, and the residual `|g| ≤ 1/256`
normalizes into a floating fraction at its own exponent, so `θ = g·π/2`
keeps full relative precision however close `x` sits to a multiple of π/2.
A degree-four minimax for sine and six Taylor terms for cosine in `θ²`
(eighteen Taylor coefficients each on the accurate leg) and a 128-entry
`sin`/`cos(j·π/256)` table recombine in `atan2q`'s frames; below 2^-8 the
argument is its own reduced angle, below 2^-57 the results are `x` and 1.

`tanq` shares that reduction and evaluates `tan θ` with a degree-six minimax
polynomial, generated independently with rminimax and certified after fixed-point
rounding. The accurate leg keeps twenty-five Taylor terms from exact Bernoulli
rationals. It then recombines by the addition formula `tan(j·π/256 + θ) =
(T_j + tan θ)/(1 − T_j·tan θ)` from a table of `tan(j·π/256)`: numerator and
denominator never cancel, and an odd quadrant just swaps them (`−cot`).  The
quotient is `atan2q`'s hardware-seeded Newton reciprocal of the denominator's
top limb plus one Newton step on the quotient itself against the full
denominator (two at 384 bits on the accurate leg), every iterate held below
the ratio so no residual goes negative.

#### Baselines

Three other binary128 implementations are within reach on a GNU/Linux box, and
they are not interchangeable:

- **[CORE-MATH]** shares metallic's ≤ 0.5 ulp contract, so it is the only fair
  performance baseline — it is the only one doing the same work.  It binds
  thirteen of the twenty functions above.
- **nightly Rust** adds no binary128 math of its own.  `f128::sin`,
  `f128::powf` and the rest are `extern "C"` calls into **glibc**'s
  `_Float128` libm (`sinf128`, `powf128`, …), and `std` documents their
  precision as "non-deterministic … varies by platform, Rust version, and can
  even differ within the same execution".  `f128::sqrt` is the one method with
  a contract (IEEE 754 `squareRoot`), and on x86-64 it does not reach glibc at
  all: the static linker binds it to `compiler_builtins`' own `sqrtf128`,
  which is why it is the one `f128::` lane within 2× of metallic instead of
  10× off.
- **libquadmath** is GCC's `__float128` runtime: a mechanical 2018 copy of
  glibc's `ldbl-128` sources (fdlibm and Moshier's Cephes) that GCC documents
  no accuracy for whatsoever.  Where it and glibc agree — bit for bit on every
  function here but `sqrt` and `hypot` — that is shared ancestry, not
  independent confirmation, and the two time within a few percent of each
  other.

`examples/f128_ulp_survey.rs` measures all three against MPFR at 300 bits:

```console
$ cargo +nightly run --release --features "f128 mpfr" --example f128_ulp_survey -- 50000
```

Maximum error in ulps over 50 000 draws per function, taken from the benches'
own bands so the accuracy population is the timed one — except `asinq`/`acosq`,
whose bench band never leaves `|x| < ½`, so the survey draws all of `[−1, 1]`
and crowds the endpoints where the reflection cancels.  Correct rounding is
`≤ 0.5`; anything above it is a wrong last bit.

| function | metallic | glibc 2.35 | libquadmath 12.3 |
|----------|:--------:|:----------:|:----------------:|
| `acosq`   | **0.500** | 1.055 | 1.055 |
| `asinq`   | **0.500** | 0.875 | 0.875 |
| `atanq`   | **0.500** | 1.068 | 1.068 |
| `atan2q`  | **0.500** | 1.541 | 1.541 |
| `cbrtq`   | **0.500** | 0.721 | 0.721 |
| `cosq`    | **0.500** | 1.387 | 1.387 |
| `exp10q`  | **0.500** | 1.712 | n/a |
| `exp2q`   | **0.500** | 0.961 | 0.961 |
| `expm1q`  | **0.500** | 1.356 | 1.356 |
| `expq`    | **0.500** | **0.500** | **0.500** |
| `hypotq`  | **0.500** | 0.533 | 1.082 |
| `log2q`   | **0.500** | 0.623 | 0.623 |
| `log10q`  | **0.500** | 0.613 | 0.613 |
| `log1pq`  | **0.500** | 2.327 | 2.327 |
| `logq`    | **0.500** | 0.501 | 0.501 |
| `powq`    | **0.500** | 0.765 | 0.765 |
| `rsqrtq`  | **0.500** | n/a | n/a |
| `sinq`    | **0.500** | 1.095 | 1.095 |
| `sqrtq`   | **0.500** | **0.500** | 0.750 |
| `tanq`    | **0.500** | 0.804 | 0.804 |

The share of draws whose last bit comes out wrong spans four orders of
magnitude: 0.006% for glibc's `logq`, ~1% for the exponentials and the sine,
3% for `tanq`, 6% for `powq` and the arc functions, 9% for `cbrtq`, 14% for
`atanq`, 17% for `exp10q`, 19% for `atan2q`, and 25% for libquadmath's `sqrtq`
and `hypotq` — neither of which is correctly rounded, while glibc's `sqrtf128`
is (IEEE mandates it) and its `hypotf128` nearly so.  These are maxima over a
random band, not a search: the adversarial lower bounds in Gladman, Innocente,
Mather and Zimmermann's [*Accuracy of Mathematical Functions*][accuracy] —
which glibc's manual now cites in place of its own deleted ulp table — are
larger still for the same glibc functions (3.51 for `log1p`, 30.3 for `pow`),
and every number above sits under them.

[accuracy]: https://inria.hal.science/hal-03141101

[complex]: https://en.cppreference.com/w/c/numeric/complex
