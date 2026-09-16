use crate::Sign;
use core::num::FpCategory;

/// Round to the nearest integer, with half-way cases rounded away from zero.
///
/// The result is exact: zeros, infinities and NaNs come back unchanged, sign
/// included, and so does every operand that is already integral.
///
/// ```
/// # #![feature(f128)]
/// assert_eq!(metallic::roundq(2.5_f128), 3.0);
/// assert_eq!(metallic::roundq(-0.5_f128), -1.0);
/// ```
#[must_use]
#[inline]
pub const fn roundq(x: f128) -> f128 {
    let bits = x.to_bits();
    let exponent = ((bits >> EXP_SHIFT) & 0x7fff) as i32 - BIAS;

    // Every operand of exponent 112 and up is an integer already, infinities
    // and NaNs (exponent 16384) included.
    if exponent >= EXP_SHIFT as i32 {
        return x;
    }
    // Below 1 the answer is the sign times 0 or 1, the half-way case `±0.5`
    // (exponent -1, like the whole binade above it) going to `±1`.
    if exponent < 0 {
        let magnitude = if exponent == -1 {
            1.0_f128.to_bits()
        } else {
            0
        };
        return f128::from_bits(bits & SIGN_MASK | magnitude);
    }

    // `0 ≤ exponent ≤ 111`: the fraction occupies the low `shift ≥ 1` bits of
    // the significand.  Adding half of the unit in the last integral place
    // carries exactly when the fraction is at least a half — ties away from
    // zero — and a carry out of the significand increments the exponent field,
    // which is the encoding of the next power of two.
    let shift = EXP_SHIFT - exponent as u32;
    let half = 1 << (shift - 1);

    f128::from_bits((bits + half) & !(half + half - 1))
}

/// Multiply `x` by 2 raised to the power `n`.
///
/// The scaling is exact unless the result overflows or falls into the
/// subnormal range, where it rounds exactly once.  Zeros, infinities and NaNs
/// come back unchanged, sign included.
///
/// ```
/// # #![feature(f128)]
/// assert_eq!(metallic::ldexpq(3.0_f128, 4), 48.0);
/// ```
#[must_use]
#[inline]
pub const fn ldexpq(x: f128, n: i32) -> f128 {
    // Scale in up to two steps per direction so the final multiply is the only
    // one that can round into the subnormal range.
    let mut x = x;
    let mut n = n;

    if n > 16383 {
        x *= exp2i(16383);
        n -= 16383;
        if n > 16383 {
            x *= exp2i(16383);
            n -= 16383;
            if n > 16383 {
                n = 16383;
            }
        }
    } else if n < -16382 {
        // -16269 = minimum normal exponent + binary128 precision.
        x *= exp2i(-16269);
        n += 16269;
        if n < -16382 {
            x *= exp2i(-16269);
            n += 16269;
            if n < -16382 {
                n = -16382;
            }
        }
    }

    x * f128::from_bits(((0x3fff + n) as u128) << EXP_SHIFT)
}

/// Decompose into a significand and an exponent.
///
/// The absolute value of the significand is in `[0.5, 1)` for nonzero finite
/// `x`, and `x == ldexpq(significand, exponent)` exactly.  Zeros, infinities
/// and NaNs come back paired with an exponent of zero.
///
/// ```
/// # #![feature(f128)]
/// assert_eq!(metallic::frexpq(48.0_f128), (0.75, 6));
/// ```
#[must_use]
#[inline]
pub const fn frexpq(x: f128) -> (f128, i32) {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return (x, 0);
    };

    let mask = f128::MIN_POSITIVE.to_bits() - 1;
    let significand = magnitude as u128 & mask | 0.5_f128.to_bits();

    (
        f128::from_bits(u128_sign_bit(sign) | significand),
        f128::MIN_EXP - 1 + (magnitude >> EXP_SHIFT) as i32,
    )
}

/// Explicitly stored significand bits in [`f128`].
pub const EXP_SHIFT: u32 = f128::MANTISSA_DIGITS - 1;

/// The sign bit of an [`f128`].
pub const SIGN_MASK: u128 = 1 << 127;

/// The exponent field of an [`f128`], which is also the bit pattern of `+∞`.
pub const EXP_MASK: u128 = 0x7fff << EXP_SHIFT;

/// The explicitly stored significand bits of an [`f128`].
pub const MANTISSA_MASK: u128 = (1 << EXP_SHIFT) - 1;

/// The implicit leading bit of a normal [`f128`] significand.
pub const IMPLICIT_BIT: u128 = 1 << EXP_SHIFT;

/// The bit that distinguishes a quiet NaN from a signaling one.
pub const QUIET_BIT: u128 = 1 << (EXP_SHIFT - 1);

/// The exponent bias of [`f128`].
pub const BIAS: i32 = 0x3fff;

/// Integer significand and unbiased exponent of a finite nonzero magnitude,
/// given as a bit pattern with the sign already stripped.
///
/// The value is `mantissa * 2^(exponent - 112)` with `mantissa` in
/// `[2^112, 2^113)`, so subnormals come back with a virtual exponent below
/// the minimum normal exponent.
#[must_use]
#[inline]
pub const fn split(bits: u128) -> (u128, i32) {
    let biased = (bits >> EXP_SHIFT) as i32;

    if biased != 0 {
        return (bits & MANTISSA_MASK | IMPLICIT_BIT, biased - BIAS);
    }
    let shift = bits.leading_zeros() - (127 - EXP_SHIFT);
    (bits << shift, 1 - BIAS - shift as i32)
}

/// Magnitude of an `f128`.
///
/// Nonzero subnormal numbers are normalized to have an implicit leading bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Magnitude {
    /// NaN.
    Nan,
    /// Infinity.
    Infinite,
    /// Zero.
    Zero,
    /// Normalized magnitude, including a virtual exponent for subnormals.
    Normalized(i128),
}

/// Break an `f128` into its sign and magnitude.
#[inline]
pub const fn normalize(x: f128) -> (Sign, Magnitude) {
    let sign = if x.is_sign_negative() {
        Sign::Negative
    } else {
        Sign::Positive
    };
    let magnitude = x.abs().to_bits() as i128;

    match x.classify() {
        FpCategory::Nan => (sign, Magnitude::Nan),
        FpCategory::Infinite => (sign, Magnitude::Infinite),
        FpCategory::Zero => (sign, Magnitude::Zero),
        FpCategory::Normal => (sign, Magnitude::Normalized(magnitude)),
        FpCategory::Subnormal => {
            const EXPONENT_DIGITS: u32 = 128 - f128::MANTISSA_DIGITS;
            let shift = magnitude.leading_zeros() as i128 - EXPONENT_DIGITS as i128;
            let magnitude = (magnitude << shift) - (shift << EXP_SHIFT);
            (sign, Magnitude::Normalized(magnitude))
        }
    }
}

const fn u128_sign_bit(sign: Sign) -> u128 {
    match sign {
        Sign::Positive => 0,
        Sign::Negative => 1 << 127,
    }
}

/// Fused multiply-add: `x * y + a` with a single rounding.
///
/// Correctly rounded where the target's `long double` is binary128 — there
/// this is glibc's `fmaf128`.
///
/// # Platform caveat
///
/// On a target whose `long double` is narrower (Apple arm64, x86 `f80`) LLVM
/// lowers the binary128 multiply-add to `fmal`, which computes the *narrow*
/// function of the low half of each argument and returns it with register
/// residue above.  The result is then wrong, not merely inaccurate, so keep
/// this off any path that has to be right everywhere.  Like the rest of
/// metallic's binary128 surface, it is supported on x86-64 GNU/Linux.
///
/// ```
/// # #![feature(f128)]
/// assert_eq!(metallic::fmaq(3.0_f128, 4.0, 5.0), 17.0);
/// ```
#[must_use]
#[allow(clippy::disallowed_methods)]
#[inline]
pub fn fmaq(x: f128, y: f128, a: f128) -> f128 {
    x.mul_add(y, a)
}

/// Const evaluation of 2<sup>`n`</sup>.
#[inline]
#[allow(clippy::cast_sign_loss)]
pub const fn exp2i(n: i64) -> f128 {
    let bits = match n + 16383 {
        32767.. => return f128::INFINITY,
        s @ 1..=32766 => (s as u128) << EXP_SHIFT,
        s @ -127..=0 => (1u128 << (EXP_SHIFT - 1)) >> -s,
        _ => 0,
    };
    f128::from_bits(bits)
}

const _: () = {
    assert!(exp2i(0).to_bits() == 1.0_f128.to_bits());
    assert!(exp2i(-16494).to_bits() == 1);
    assert!(exp2i(16384).is_infinite());
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_magnitudes() {
        assert_eq!(split(1.0_f128.to_bits()), (IMPLICIT_BIT, 0));
        assert_eq!(split(f128::MIN_POSITIVE.to_bits()), (IMPLICIT_BIT, -16382));
        assert_eq!(split(1), (IMPLICIT_BIT, -16494));
        assert_eq!(split(MANTISSA_MASK), (MANTISSA_MASK << 1, -16383));
    }

    #[test]
    fn frexp_round_trip() {
        for x in [
            f128::from_bits(1),
            f128::MIN_POSITIVE,
            1.0,
            f128::MAX,
            -f128::from_bits(1),
        ] {
            let (fraction, exponent) = frexpq(x);
            assert_eq!(ldexpq(fraction, exponent).to_bits(), x.to_bits());
        }
    }
}
