//! Real functions for `f128`.
#![allow(clippy::pedantic, clippy::approx_constant)]
#![warn(clippy::unreadable_literal)]

mod asin;
mod atan2;
mod atan2_tables;
mod cbrt_tables;
mod exp;
mod exp_tables;
mod hypot;
mod log;
mod log10_tables;
mod log2_tables;
mod log_tables;
#[allow(dead_code)]
mod misc;
#[cfg(feature = "mpfr")]
#[doc(hidden)]
pub mod mpfr;
mod roots;
mod rsqrt_tables;
mod tan;
mod trig;
mod trig_tables;
#[allow(dead_code)]
mod uint;

pub use asin::{acosq, asinq};
pub use atan2::{atan2q, atanq};
pub use exp::{exp2q, exp10q, expm1q, expq};
pub use hypot::hypotq;
pub use log::{log1pq, log2q, log10q, logq};
pub use roots::{cbrtq, rsqrtq, sqrtq};
pub use tan::tanq;
pub use trig::{cosq, sinq};

// Internal binary128 primitives shared by the function implementations.
#[allow(unused_imports)]
pub use misc::{
    BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, MANTISSA_MASK, Magnitude, QUIET_BIT, SIGN_MASK, exp2i,
    fma128, frexp, ldexp, normalize, split,
};
