mod kernel;
use core::num::NonZeroU32;

/// Explicitly stored significand bits
pub const EXP_SHIFT: u32 = f32::MANTISSA_DIGITS - 1;

/// Normalize nonzero magnitude
#[allow(clippy::cast_possible_wrap)]
const fn normalize(x: NonZeroU32) -> i32 {
    let x = x.get() as i32;

    if x < 0x0080_0000 {
        let shift = x.leading_zeros() as i32 - 8;
        (x << shift) - (shift << EXP_SHIFT)
    } else {
        x
    }
}

/// The least number greater than `x`
///
/// This is a less careful version of [`f32::next_up`] regarding subnormal
/// numbers.  This function is useful until `f32::next_up` gets stabilized.
#[must_use]
pub fn next_up(x: f32) -> f32 {
    if x.is_nan() || x == f32::INFINITY {
        x
    } else if x == 0.0 {
        f32::from_bits(1)
    } else if x.is_sign_negative() {
        f32::from_bits(x.to_bits() - 1)
    } else {
        f32::from_bits(x.to_bits() + 1)
    }
}

/// The greatest number less than `x`
///
/// This is a less careful version of [`f32::next_down`] regarding subnormal
/// numbers.  This function is useful until `f32::next_down` gets stabilized.
#[must_use]
pub fn next_down(x: f32) -> f32 {
    if x.is_nan() || x == f32::NEG_INFINITY {
        x
    } else if x == 0.0 {
        f32::from_bits(0x8000_0001)
    } else if x.is_sign_negative() {
        f32::from_bits(x.to_bits() + 1)
    } else {
        f32::from_bits(x.to_bits() - 1)
    }
}

/// The exponential function
#[must_use]
pub fn exp(x: f32) -> f32 {
    use core::f32::consts::LN_2;
    use core::f64::consts;

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 * LN_2 {
        return 0.0;
    }

    #[allow(clippy::cast_precision_loss)]
    if x > f32::MAX_EXP as f32 * LN_2 {
        return f32::INFINITY;
    }

    let x = f64::from(x);
    let n = (x * consts::LOG2_E).round_ties_even();
    let x = n.mul_add(-consts::LN_2, x);
    let y = kernel::exp(x).mul_add(x, 1.0);

    #[allow(clippy::cast_possible_truncation)]
    return kernel::fast_ldexp(y, n as i64) as f32;
}

/// Multiply `x` by 2 raised to the power of `n`
#[must_use]
pub fn ldexp(x: f32, n: i32) -> f32 {
    const MIN_EXP: i32 = f64::MIN_EXP - 1;
    const MAX_EXP: i32 = f64::MAX_EXP;

    let coefficient = match n {
        ..MIN_EXP => 0.5 * f64::MIN_POSITIVE,

        #[allow(clippy::cast_sign_loss)]
        n @ MIN_EXP..MAX_EXP => f64::from_bits(((MAX_EXP - 1 + n) as u64) << crate::f64::EXP_SHIFT),

        MAX_EXP.. => f64::MAX,
    };

    #[allow(clippy::cast_possible_truncation)]
    return (f64::from(x) * coefficient) as f32;
}

/// Decompose into a significand and an exponent
///
/// The absolute value of the significand is in the range of [0.5, 1) for
/// nonzero finite `x` for historical reasons.  This function also explains how
/// [`f32::MAX_EXP`] and [`f32::MIN_EXP`] are defined.
#[must_use]
pub fn frexp(x: f32) -> (f32, i32) {
    let Some(magnitude) = NonZeroU32::new(x.abs().to_bits()) else {
        return (x, 0);
    };

    if magnitude.get() >= f32::INFINITY.to_bits() {
        return (x, 0);
    }

    let magnitude = normalize(magnitude);
    let mask = f32::MIN_POSITIVE.to_bits() - 1;

    #[allow(clippy::cast_sign_loss)]
    let significand = magnitude as u32 & mask | 0.5f32.to_bits();

    (
        f32::from_bits(significand).copysign(x),
        f32::MIN_EXP - 1 + (magnitude >> EXP_SHIFT),
    )
}
