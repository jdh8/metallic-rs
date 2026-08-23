//! Real functions for `f128`.
#![allow(clippy::pedantic, clippy::approx_constant)]
#![warn(clippy::unreadable_literal)]

mod exp;
mod exp_tables;
mod hypot;
#[allow(dead_code)]
mod misc;
#[cfg(feature = "mpfr")]
#[doc(hidden)]
pub mod mpfr;
mod roots;
#[allow(dead_code)]
mod uint;

pub use exp::{exp2q, exp10q, expm1q, expq};
pub use hypot::hypotq;
pub use roots::{cbrtq, rsqrtq, sqrtq};

// Internal binary128 primitives shared by the function implementations.
#[allow(unused_imports)]
pub use misc::{
    BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, MANTISSA_MASK, Magnitude, QUIET_BIT, SIGN_MASK, exp2i,
    fma128, frexp, ldexp, normalize, split,
};
