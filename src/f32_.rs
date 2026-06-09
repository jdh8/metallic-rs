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

use crate::Sign;
use core::f32;
use core::num::FpCategory;

/// Higher part of ln(2) whose lowest 14 bits are zero
const LN_2_HI: f64 = 0.693_147_180_560_117_7;

/// Lower part of ln(2)
///
/// To be precise, this is the `f64` closest to ln(2) - [`LN_2_HI`].
const LN_2_LO: f64 = -1.723_944_452_561_483_5e-13;

const _: () = assert!(LN_2_HI + LN_2_LO == core::f64::consts::LN_2);

/// Explicitly stored significand bits in [`prim@f32`]
///
/// This constant is usually used as a shift to access the exponent bits.
pub const EXP_SHIFT: u32 = f32::MANTISSA_DIGITS - 1;

/// Magnitude of `f32`
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
    /// The layout of the bits is the same as a normal positive `f32`.  For
    /// subnormal numbers, the stored exponent becomes zero or negative while
    /// the significand is normalized to have an implicit leading bit.
    Normalized(i32),
}

/// Break a `f32` into its sign and magnitude
#[inline]
const fn normalize(x: f32) -> (Sign, Magnitude) {
    let sign = if x.is_sign_negative() {
        Sign::Negative
    } else {
        Sign::Positive
    };
    let magnitude = x.abs().to_bits() as i32;

    match x.classify() {
        FpCategory::Nan => (sign, Magnitude::Nan),
        FpCategory::Infinite => (sign, Magnitude::Infinite),
        FpCategory::Zero => (sign, Magnitude::Zero),
        FpCategory::Normal => (sign, Magnitude::Normalized(magnitude)),
        FpCategory::Subnormal => {
            const EXPONENT_DIGITS: u32 = 32 - f32::MANTISSA_DIGITS;
            let shift = magnitude.leading_zeros() as i32 - EXPONENT_DIGITS as i32;
            let magnitude = (magnitude << shift) - (shift << EXP_SHIFT);
            (sign, Magnitude::Normalized(magnitude))
        }
    }
}

pub use atan::{acos, asin, atan, atan2};
pub use erf::{erf, erfc};
pub use exp::{exp, exp_m1, exp2, exp10, frexp, ldexp};
pub use gamma::{lgamma, tgamma};
pub use hyp::{acosh, asinh, atanh, cosh, sinh, tanh};
pub use log::{ln, ln_1p, log2, log10};
pub use misc::{cbrt, hypot, round};
pub use pow::powf;
pub use trig::{cos, sin, sin_cos, tan};
