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

pub use atan::{acosf, acospif, asinf, asinpif, atan2f, atan2pif, atanf, atanpif};
pub use erf::{erfcf, erff};
pub use exp::{exp2f, exp10f, expf, expm1f, frexpf, ldexpf};
pub use gamma::{lgammaf, tgammaf};
pub use hyp::{acoshf, asinhf, atanhf, coshf, sinhf, tanhf};
pub use log::{log1pf, log2f, log10f, logf};
pub use misc::{cbrtf, fmaf, hypotf, roundf, rsqrtf};
pub use pow::powf;
pub use trig::{cosf, cospif, sincosf, sinf, sinpif, tanf, tanpif};

// Internal helpers shared across this module's submodules (reached as `super::…`).
pub use misc::{EXP_SHIFT, LN_2_HI, LN_2_LO, Magnitude, normalize, u32_sign_bit};
