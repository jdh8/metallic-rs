#![allow(clippy::pedantic, clippy::approx_constant)]
#![warn(clippy::unreadable_literal)]

mod atan;
mod dint;
mod dint_consts;
pub(crate) mod double;
mod exp;
mod hyp;
mod log;
mod misc;
pub(crate) mod pow;
mod trig;
use crate::Sign;
use core::{f64, num::FpCategory};

/// Explicitly stored significand bits in [`prim@f64`]
///
/// This constant is usually used as a shift to access the exponent bits.
pub const EXP_SHIFT: u32 = f64::MANTISSA_DIGITS - 1;

/// Magnitude of `f64`
///
/// Nonzero subnormal numbers are normalized to have an implicit leading bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Magnitude {
    /// NaN, see [`FpCategory::Nan`]
    Nan,

    /// Infinity, see [`FpCategory::Infinite`]
    Infinite,

    /// Zero, see [`FpCategory::Zero`]
    ///
    /// Zero cannot be normalized.  A normalized magnitude has an implicit
    /// leading bit.
    Zero,

    /// Normalized magnitude
    ///
    /// The layout of the bits is the same as a normal positive `f64`.  For
    /// subnormal numbers, the stored exponent becomes zero or negative while
    /// the significand is normalized to have an implicit leading bit.
    Normalized(i64),
}

/// Break a `f64` into its sign and magnitude
#[inline]
const fn normalize(x: f64) -> (Sign, Magnitude) {
    let sign = if x.is_sign_negative() {
        Sign::Negative
    } else {
        Sign::Positive
    };
    let magnitude = x.abs().to_bits() as i64;

    match x.classify() {
        FpCategory::Nan => (sign, Magnitude::Nan),
        FpCategory::Infinite => (sign, Magnitude::Infinite),
        FpCategory::Zero => (sign, Magnitude::Zero),
        FpCategory::Normal => (sign, Magnitude::Normalized(magnitude)),
        FpCategory::Subnormal => {
            const EXPONENT_DIGITS: u32 = 64 - f64::MANTISSA_DIGITS;
            let shift = magnitude.leading_zeros() as i64 - EXPONENT_DIGITS as i64;
            let magnitude = (magnitude << shift) - (shift << EXP_SHIFT);
            (sign, Magnitude::Normalized(magnitude))
        }
    }
}

pub use atan::{acos, asin, atan, atan2};
pub use exp::{exp, exp10, exp2, exp_m1};
pub use hyp::{acosh, asinh, atanh, cosh, sinh, tanh};
pub use log::{ln, ln_1p, log, log10, log2};
pub use misc::{cbrt, frexp, hypot, ldexp, round};
pub use pow::powf;
pub use trig::{cos, sin, sin_cos, tan};

pub(crate) use log::ln_dd;
