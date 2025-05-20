#![allow(clippy::pedantic)]
#![warn(clippy::unreadable_literal)]

mod kernel;
use crate::Sign;
use core::{f64, num::FpCategory};
use kernel::Sum;

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

/// Rounds half-way cases away from zero
#[must_use]
#[inline]
pub fn round(x: f64) -> f64 {
    let r = x.abs();
    let i = r.trunc();

    (i + f64::from(r - i >= 0.5)).copysign(x)
}

/// The cube root
#[must_use]
#[inline]
pub fn cbrt(x: f64) -> f64 {
    let (x, coefficient) = match x.abs() {
        0.0 => return x,
        0.0..1e-200 => (crate::exp2i(999) * x, crate::exp2i(-333)),
        1e-200..=1e200 => (x, 1.0),
        1e200..f64::INFINITY => (crate::exp2i(-999) * x, crate::exp2i(333)),
        _ => return x,
    };

    let sign_bit = x.to_bits() >> 63 << 63;
    let magnitude = 0x2A9F_7AF1_96E8_E6E8 + x.abs().to_bits() / 3;
    let y = f64::from_bits(sign_bit | magnitude);
    let y = crate::mul_add(1.0 / 3.0, x / (y * y) - y, y);
    let y = crate::mul_add(1.0 / 3.0, x / (y * y) - y, y);
    let y = y * (0.5 + 1.5 * x / crate::mul_add(2.0 * y, y * y, x));

    let quotient = Sum::from_quotient(x, y) / y;
    let sum = kernel::fast_sum(2.0 * y, quotient.high);
    let sum = Sum {
        high: sum.high,
        low: quotient.low + sum.low,
    } / 3.0;

    coefficient * (sum.high + sum.low)
}
