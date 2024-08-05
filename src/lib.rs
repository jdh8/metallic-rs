//! C math functions from scratch for Rust
//!
//! This library is a successor to [Metallic], my C library for WebAssembly
//! started in 2017.  Its most wanted feature turned out to be math functions
//! I wrote from scratch, so I decided to rewrite them in Rust.
//!
//! [Metallic]: https://github.com/jdh8/metallic
//!
//! Assumptions
//! -----------
//! C libraries tend to have strict yet obsolete assumptions on math
//! functions. For example, `float` functions dare not use `double`
//! instructions for fear that the host does not support them.  In this
//! library, I assume all Rust primitive types are IEEE 754 compliant and
//! native to the host.  In other words, I assume the following instructions
//! are available to floating-point types:
//!
//! - Addition, subtraction, multiplication, division
//! - Square root ([`f32::sqrt`])
//! - Fused multiply-add ([`f32::mul_add`])
//! - Rounding instructions such as [`f32::trunc`]
//!
//! The assumptions beyond the four basic arithmetic operations creates
//! dependency on the [Rust standard library][std].  This library is not
//! intended for embedded systems.
//!
//! Goals
//! -----
//! - The functions should be faithfully rounded (error < 1 ulp).
//! - The functions should be about as fast as the system library.
//! - Try to make `f32` functions faster than the system library.
//! - Avoid lookup tables to reduce memory usage, especially on WebAssembly.
//!     + This goal is not as important as the others.  For example, a lookup
//!       table for trigonometric functions is required to achieve faithful
//!       rounding.  See [Payne&ndash;Hanek
//!       reduction](https://doi.org/10.1145/1057600.1057602) for more
//!       details.
//!
//! ### Non-goals
//! - I skip rounding functions such as `rint`, `round`, and `trunc` because
//!     + They are likely to be a single instruction on modern CPUs.
//!     + Rust already provides [`f32::round_ties_even`], [`f32::round`],
//!       [`f32::trunc`], etc.
//!     + Their software implementations are slow and tedious, unlike
//!       [`f32::abs`].

#![warn(missing_docs)]

/// Real functions for `f32`s
pub mod f32;
