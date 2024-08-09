mod kernel;
use core::num::FpCategory;

/// Explicitly stored significand bits in [`f32`]
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
#[allow(clippy::cast_possible_wrap)]
#[inline]
fn normalize(x: f32) -> (bool, Magnitude) {
    let sign = x.is_sign_negative();
    let magnitude = x.abs().to_bits() as i32;

    match x.classify() {
        FpCategory::Nan => (sign, Magnitude::Nan),
        FpCategory::Infinite => (sign, Magnitude::Infinite),
        FpCategory::Zero => (sign, Magnitude::Zero),
        FpCategory::Normal => (sign, Magnitude::Normalized(magnitude)),
        FpCategory::Subnormal => {
            let shift = magnitude.leading_zeros() as i32 - 8;
            (
                sign,
                Magnitude::Normalized((magnitude << shift) - (shift << EXP_SHIFT)),
            )
        }
    }
}

/// Fast multiply-add
///
/// This function picks the faster way to compute `x * y + a` depending on the
/// target architecture.  The FMA instruction is used if available.  Otherwise,
/// it falls back to `x * y + a` that is faster but gives less accurate results
/// than [`f32::mul_add`]
#[must_use]
#[inline]
pub fn mul_add(x: f32, y: f32, a: f32) -> f32 {
    #[cfg(target_feature = "fma")]
    return x.mul_add(y, a);

    #[cfg(not(target_feature = "fma"))]
    return x * y + a;
}

/// The least number greater than `x`
///
/// This is a less careful version of [`f32::next_up`] regarding subnormal
/// numbers.  This function is useful until `f32::next_up` gets stabilized.
#[must_use]
#[inline]
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
#[inline]
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

/// The cube root
#[must_use]
#[inline]
pub fn cbrt(x: f32) -> f32 {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return x;
    };

    // SAFETY: the minimum magnitude is -0x0B00_0000, so the cast integer is
    // always positive
    #[allow(clippy::cast_sign_loss)]
    let magnitude = (0x2A51_2CE3 + magnitude / 3) as u32;

    let iter = |y: f32| mul_add(3.0f32.recip(), x / (y * y) - y, y);
    iter(iter(iter(f32::from_bits(
        u32::from(sign) << 31 | magnitude,
    ))))
}

/// The exponential function
#[must_use]
#[inline]
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
    let x = crate::f64::mul_add(n, -consts::LN_2, x);
    let y = crate::f64::mul_add(kernel::exp(x), x, 1.0);

    #[allow(clippy::cast_possible_truncation)]
    return kernel::fast_ldexp(y, n as i64) as f32;
}

/// Compute `2^x`
#[must_use]
#[inline]
pub fn exp2(x: f32) -> f32 {
    const P: [f32; 6] = [
        6.931_472e-1,
        2.402_265e-1,
        5.550_357e-2,
        9.618_031e-3,
        1.339_086_7e-3,
        1.546_973_5e-4,
    ];

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 {
        return 0.0;
    }

    #[allow(clippy::cast_precision_loss)]
    if x > f32::MAX_EXP as f32 {
        return f32::INFINITY;
    }

    let n = x.round_ties_even();
    let x = x - n;
    let y = mul_add(P[5], x, P[4]);
    let y = mul_add(y, x, P[3]);
    let y = mul_add(y, x, P[2]);
    let y = mul_add(y, x, P[1]);
    let y = mul_add(y, x, P[0]);
    let y = mul_add(y, x, 1.0);

    #[allow(clippy::cast_possible_truncation)]
    return kernel::fast_ldexp(f64::from(y), n as i64) as f32;
}

/// Compute `exp(x) - 1` accurately especially for small `x`
#[must_use]
#[inline]
pub fn exp_m1(x: f32) -> f32 {
    use core::f32::consts::LN_2;
    use core::f64::consts;

    #[allow(clippy::cast_precision_loss)]
    if x < f32::MANTISSA_DIGITS as f32 * -LN_2 {
        return -1.0;
    }

    #[allow(clippy::cast_precision_loss)]
    if x > f32::MAX_EXP as f32 * LN_2 {
        return f32::INFINITY;
    }

    let x = f64::from(x);
    let n = (x * consts::LOG2_E).round_ties_even() + 0.0;
    let x = crate::f64::mul_add(n, -consts::LN_2, x);
    let y = kernel::exp(x);

    if n == 0.0 {
        #[allow(clippy::cast_possible_truncation)]
        return (x * y) as f32;
    }

    #[allow(clippy::cast_possible_truncation)]
    return (kernel::fast_ldexp(crate::f64::mul_add(x, y, 1.0), n as i64) - 1.0) as f32;
}

/// Multiply `x` by 2 raised to the power of `n`
#[must_use]
#[inline]
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
#[inline]
pub fn frexp(x: f32) -> (f32, i32) {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return (x, 0);
    };

    let mask = f32::MIN_POSITIVE.to_bits() - 1;

    #[allow(clippy::cast_sign_loss)]
    let significand = magnitude as u32 & mask | 0.5f32.to_bits();

    (
        f32::from_bits(u32::from(sign) << 31 | significand),
        f32::MIN_EXP - 1 + (magnitude >> EXP_SHIFT),
    )
}
