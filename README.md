Math functions from scratch
===========================
[![Build status](https://github.com/jdh8/metallic-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jdh8/metallic-rs)
[![Crates.io](https://img.shields.io/crates/v/metallic.svg)](https://crates.io/crates/metallic)
[![Documentation](https://docs.rs/metallic/badge.svg)](https://docs.rs/metallic)

This library is a successor to [Metallic], my C library for WebAssembly
started in 2017.  Its most wanted feature turned out to be math functions I
wrote from scratch, so I decided to rewrite them in Rust.

[Metallic]: https://github.com/jdh8/metallic

Enable [fused multiply-add][fma] for best performance!
------------------------------------------------------
This crate extensively uses the fused multiply-add instruction if available.
Sadly, Rust does not enable it in the default `generic` target.  To achieve best
performance, add the following to your `.cargo/config.toml` either in **your**
project or home directory:

```toml
[build]
rustflags = ["-Ctarget-cpu=native"]
```

[fma]: https://en.wikipedia.org/wiki/Multiply%E2%80%93accumulate_operation

Assumptions
-----------
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
Rust.  It is also mostly unused in C and C++ because it requires
`#pragma STDC FENV_ACCESS ON` and compiler support.

Goals
-----
- The functions should be faithfully rounded (error < 1 ulp).
- The functions should be about as fast as the system library.
- Try to make `f32` functions faster than the system library.
- Avoid lookup tables to reduce memory usage, especially on WebAssembly.
    + This goal is not as important as the others.  For example, a lookup
      table for trigonometric functions is required to achieve faithful
      rounding.  See [Payne&ndash;Hanek reduction](https://doi.org/10.1145/1057600.1057602)
      for more details.

### Non-goals
- I skip rounding functions such as `rint`, `round`, and `trunc` because
    + They are likely to be a single instruction on modern CPUs.
    + Rust already provides
      [`f32::round_ties_even`](https://doc.rust-lang.org/std/primitive.f32.html#method.round_ties_even),
      [`f32::round`](https://doc.rust-lang.org/std/primitive.f32.html#method.round),
      [`f32::trunc`](https://doc.rust-lang.org/std/primitive.f32.html#method.trunc),
      etc.
    + Their software implementations are slow and tedious, unlike `fabs`.

Milestones
----------
- [ ] Real `f32`/`float` functions in [`<math.h>`][math]
    - [x] Exponential functions
    - [x] Logarithm with constant base
    - [x] Power and logarithm with arbitrary base
    - [ ] Trigonometric and hyperbolic functions
    - [x] Miscellaneous elementary functions
    - [ ] Non-elementary functions (optional)
- [ ] Complex `f32`/`float` functions in [`<complex.h>`][complex]
- [ ] Real `f64`/`double` functions in [`<math.h>`][math]
- [ ] Complex `f64`/`double` functions in [`<complex.h>`][complex]

[math]: https://en.cppreference.com/w/c/numeric/math
[complex]: https://en.cppreference.com/w/c/numeric/complex