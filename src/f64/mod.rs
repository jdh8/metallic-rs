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

    let four_thirds = Sum::from_quotient(4.0, 3.0);
    let nx_thirds = Sum::from_quotient(x, -3.0);
    let newton = |y: f64| {
        let yy = y * y;
        crate::mul_add(four_thirds.high, y, nx_thirds.high * yy * yy)
    };

    let sign_bit = x.to_bits() >> 63 << 63;
    let magnitude = 0x553E_C750_CF65_7065 - x.abs().to_bits() / 3;
    let y = f64::from_bits(sign_bit | magnitude);
    let y = newton(newton(newton(newton(y))));

    let yy = Sum::from_product(y, y);
    let lhs = four_thirds * y;
    let rhs = nx_thirds * yy * yy;

    let y = kernel::fast_sum(lhs.high, rhs.high);
    let y = Sum {
        high: y.high,
        low: lhs.low + rhs.low + y.low,
    };
    let yy = y * y;

    yy.high.mul_add(x, yy.low * x) * coefficient
}
