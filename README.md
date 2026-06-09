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
`f64`, and the `f` suffix marks the `f32` variant, so the crate is a drop-in
replacement for `libm` and `core-math`.

```rust
assert_eq!(metallic::exp(0.0), 1.0); // f64
assert_eq!(metallic::expf(0.0_f32), 1.0); // f32
```

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

## Using faster functions from [CORE-MATH]

I struggle to make some functions faster than [CORE-MATH].  You can enable the
[`core-math`](crate) feature in your `Cargo.toml`:

```toml
[dependencies]
metallic = { version = "0.2.0", features = ["core-math"] }
```

This would replace the following functions with those from
[`core-math`][crate]:

- `powf`, which is notoriously hard to round correctly

[CORE-MATH]: https://core-math.gitlabpages.inria.fr/
[crate]: https://crates.io/crates/core-math

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

- I skip rounding functions such as `rint`, `round`, and `trunc` because
  - They are likely to be a single instruction on modern CPUs.
  - Rust already provides
    [`f32::round_ties_even`](https://doc.rust-lang.org/std/primitive.f32.html#method.round_ties_even),
    [`f32::round`](https://doc.rust-lang.org/std/primitive.f32.html#method.round),
    [`f32::trunc`](https://doc.rust-lang.org/std/primitive.f32.html#method.trunc),
    etc.
  - Their software implementations are slow and tedious, unlike `fabs`.

## Milestones

- [x] Real `f32`/`float` functions in [`<math.h>`][math]
  - [x] Exponential functions
  - [x] Logarithm with constant base
  - [x] Power with arbitrary base
  - [x] Trigonometric and hyperbolic functions
  - [x] Miscellaneous elementary functions
  - [x] Non-elementary functions (optional)
- [ ] Complex `f32`/`float` functions in [`<complex.h>`][complex]
- [x] Real `f64`/`double` functions in [`<math.h>`][math]
  - [x] Exponential functions
  - [x] Logarithm with constant base
  - [x] Power with arbitrary base
  - [x] Trigonometric and hyperbolic functions
  - [x] Miscellaneous elementary functions
  - [x] Non-elementary functions (optional)
- [ ] Complex `f64`/`double` functions in [`<complex.h>`][complex]

[math]: https://en.cppreference.com/w/c/numeric/math
[complex]: https://en.cppreference.com/w/c/numeric/complex
