//! Real functions for `f32`, implemented in terms of `f64`.
#![allow(clippy::pedantic, clippy::approx_constant)]
#![warn(clippy::unreadable_literal)]

mod atan;
mod erf;
mod exp;
mod gamma;
mod hyp;
mod log;
mod misc;
mod pow;
mod trig;

pub use atan::{acos, asin, atan, atan2};
pub use erf::{erf, erfc};
pub use exp::{exp, exp_m1, exp2, exp10, frexp, ldexp};
pub use gamma::{lgamma, tgamma};
pub use hyp::{acosh, asinh, atanh, cosh, sinh, tanh};
pub use log::{ln, ln_1p, log2, log10};
pub use misc::{cbrt, fmaf, hypot, round};
pub use pow::powf;
pub use trig::{cos, sin, sin_cos, tan};

// Internal helpers shared across this module's submodules (reached as `super::…`).
pub use misc::{EXP_SHIFT, LN_2_HI, LN_2_LO, Magnitude, normalize, u32_sign_bit};
