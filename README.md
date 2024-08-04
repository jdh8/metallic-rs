metallic-rs
===========
C math functions from scratch for Rust

This library is a successor to [Metallic], my C library for WebAssembly
started in 2017.  Its most wanted feature turned out to be math functions I
wrote from scratch, so I decided to rewrite them in Rust.

[Metallic]: https://github.com/jdh8/metallic

C libraries tend to have strict yet obsolete assumptions on math functions.

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
- [ ] Complex `f32`/`float` functions in [`<complex.h>`][complex]
- [ ] Real `f64`/`double` functions in [`<math.h>`][math]
- [ ] Complex `f64`/`double` functions in [`<complex.h>`][complex]

[math]: https://en.cppreference.com/w/c/numeric/math
[complex]: https://en.cppreference.com/w/c/numeric/complex