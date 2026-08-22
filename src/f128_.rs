//! Real functions for `f128`.
#![allow(clippy::pedantic, clippy::approx_constant)]
#![warn(clippy::unreadable_literal)]

#[allow(dead_code)]
mod misc;
#[cfg(feature = "mpfr")]
#[doc(hidden)]
pub mod mpfr;
mod roots;
#[allow(dead_code)]
mod uint;

pub use roots::{cbrtq, rsqrtq, sqrtq};

// Internal binary128 primitives shared by the function implementations.
#[allow(unused_imports)]
pub use misc::{EXP_SHIFT, Magnitude, exp2i, fma128, frexp, ldexp, normalize};
#[allow(unused_imports)]
pub use uint::{mhi, wmul};
