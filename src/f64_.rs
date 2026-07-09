//! Real functions for `f64`.
#![allow(clippy::pedantic, clippy::approx_constant)]
#![warn(clippy::unreadable_literal)]

mod atan;
mod dint;
mod dint_consts;
pub mod double;
mod erf;
mod exp;
mod gamma;
mod hyp;
mod log;
mod misc;
pub mod pow;
mod pow_accurate;
mod pow_consts;
mod qint;
pub mod trig;

pub use atan::{acos, acospi, asin, asinpi, atan, atan2, atan2pi, atanpi};
pub use erf::{erf, erfc};
pub use exp::{exp, exp2, exp2m1, exp10, exp10m1, expm1};
pub use gamma::{lgamma, tgamma};
pub use hyp::{acosh, asinh, atanh, cosh, sinh, tanh};
pub use log::{ln_dd, ln_dd_fast, ln_fast, ln_fast_scaled, log, log1p, log2, log10};
pub use misc::{cbrt, fma, frexp, hypot, ldexp, round, rsqrt};
pub use trig::{cos, cospi, sin, sincos, sinpi, tan, tanpi};

// Internal primitives/helpers shared across this module's submodules and the
// crate root (reached as `super::…` / `crate::…`).
pub use misc::{EXP_SHIFT, Magnitude, exp2i, fast_mul_add, normalize};
