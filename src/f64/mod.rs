#![allow(clippy::pedantic)]
#![warn(clippy::unreadable_literal)]

mod exp_consts;
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

/// The exponential function
#[must_use]
#[inline]
pub fn exp(x: f64) -> f64 {
    use exp_consts::{EXP2_TABLE, EXP_R_COEFFS, LN2_OVER_N_HI, LN2_OVER_N_LO};

    /// Table size, so `exp(x) = 2`<sup>`q`</sup>` · 2`<sup>`j/N`</sup>` · exp(r)`
    const N: i64 = 128;

    /// `N / ln(2)`, the scale that maps `x` to the reduction index
    const N_OVER_LN2: f64 = 184.6649652337873;

    if x.is_nan() {
        return x;
    }

    // `ln(f64::MAX)` and the threshold below which `exp` rounds to zero
    if x >= 709.782712893384 {
        return f64::INFINITY;
    }
    if x <= -745.133219101941 {
        return 0.0;
    }

    // Argument reduction: n = round(N·x / ln2), so r = x − n·ln2/N lies in
    // [−ln2/2N, ln2/2N] ≈ [−0.0027, 0.0027].
    let scaled = (x * N_OVER_LN2).round_ties_even();

    // SAFETY: `|x| < 746`, so `|scaled| < 2^18`.
    let n = unsafe { scaled.to_int_unchecked::<i64>() };
    let j = (n & (N - 1)) as usize;
    let q = n >> 7;

    // r as a double-double.  `scaled · LN2_OVER_N_HI` is exact because the high
    // word has 17 trailing zero bits, and the low word recovers the tail.
    let a = scaled.mul_add(-LN2_OVER_N_HI, x);
    let r = Sum::from_sum(a, scaled * -LN2_OVER_N_LO);

    // exp(r) by double-double Horner over the degree-8 minimax polynomial.
    let (high, low) = EXP_R_COEFFS[EXP_R_COEFFS.len() - 1];
    let mut acc = Sum { high, low };

    for &(high, low) in EXP_R_COEFFS[..EXP_R_COEFFS.len() - 1].iter().rev() {
        acc = acc * r + Sum { high, low };
    }

    // exp(x) = 2^q · 2^(j/N) · exp(r); fold the table entry in and normalize.
    let (high, low) = EXP2_TABLE[j];
    let scaled = Sum { high, low } * acc;
    let product = kernel::fast_sum(scaled.high, scaled.low);

    // `2^(j/N) · exp(r)` lies in [0.997, 2.005); fold its exponent into `q` so the
    // mantissa is in [1, 2).  Then the result is subnormal exactly when `q < −1022`,
    // and the integer-grid shift below stays within an exact `i64`.
    let (product, q) = if product.high < 1.0 {
        (product * 2.0, q - 1)
    } else if product.high >= 2.0 {
        (product * 0.5, q + 1)
    } else {
        (product, q)
    };

    if q >= -1022 {
        // Normal result: scaling by 2^q is exact, so one rounding of the pair.
        return kernel::fast_ldexp(product.high + product.low, q);
    }

    // Subnormal result: rounding the pair to `f64` and then scaling would round
    // twice.  Instead round the double-double on the integer grid at scale
    // 2^-1074 (the subnormal ulp): `m = (high + low) · 2^(q + 1074)` lies in
    // [0, 2^52], round it once to an integer, then `n · 2^-1074` is exact.
    let shift = q + 1074;
    let high = kernel::fast_ldexp(product.high, shift);
    let low = kernel::fast_ldexp(product.low, shift);

    // `high` may carry a half-integer resolution at this scale, so `high + low`
    // would discard the fine part of `low`.  Round `high`, then correct with the
    // exact residual `(high − n0) + low`.
    let n0 = high.round_ties_even();
    let n = n0 + ((high - n0) + low).round_ties_even();

    // 2^-1074 is the smallest positive subnormal, i.e. `f64::from_bits(1)`.
    n * f64::from_bits(1)
}
