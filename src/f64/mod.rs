#![allow(clippy::pedantic)]
#![warn(clippy::unreadable_literal)]

mod kernel;
use crate::Sign;
use core::num::FpCategory;
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
    match x.abs() {
        5e-324 => return 1.703_183_936_003_260_3e-108_f64.copysign(x),
        1.797_693_134_862_315_7e308 => return 5.643_803_094_122_362e102_f64.copysign(x),
        _ => (),
    }

    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return x;
    };

    let magnitude = (0x2A9F_7AF1_96E8_E6E8 + magnitude / 3) as u64;
    let y = f64::from_bits(crate::u64_sign_bit(sign) | magnitude);
    let y = y * crate::mul_add(x / crate::mul_add(2.0 * y, y * y, x), 1.5, 0.5);
    let y = y * crate::mul_add(x / crate::mul_add(2.0 * y, y * y, x), 1.5, 0.5);
    let y = y * crate::mul_add(x / crate::mul_add(2.0 * y, y * y, x), 1.5, 0.5);

    let quotient = Sum::from_quotient(x, y) / y;
    let sum = kernel::fast_sum(2.0 * y, quotient.high);
    let sum = Sum {
        high: sum.high,
        low: quotient.low + sum.low,
    } / 3.0;

    sum.high + sum.low
}
