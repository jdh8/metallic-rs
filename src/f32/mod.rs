#![allow(clippy::pedantic)]
#![warn(clippy::unreadable_literal)]

mod kernel;
use core::cmp::Ordering;
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
            let magnitude = (magnitude << shift) - (shift << EXP_SHIFT);
            (sign, Magnitude::Normalized(magnitude))
        }
    }
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

/// Rounds half-way cases away from zero
#[must_use]
#[inline]
pub fn round(x: f32) -> f32 {
    let r = x.abs();
    let i = r.trunc();

    (i + f32::from(r - i >= 0.5)).copysign(x)
}

/// The cube root
#[must_use]
#[inline]
pub fn cbrt(x: f32) -> f32 {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return x;
    };

    let magnitude = (0x2A51_2CE3 + magnitude / 3) as u32;
    let x: f64 = x.into();
    let y: f64 = f32::from_bits(u32::from(sign) << 31 | magnitude).into();
    let y = y * (0.5 + 1.5 * x / crate::mul_add(2.0 * y, y * y, x));
    let y = y * (0.5 + 1.5 * x / crate::mul_add(2.0 * y, y * y, x));

    y as f32
}

/// Hypotenuse of a right-angled triangle with sides `x` and `y`
#[must_use]
#[inline]
pub fn hypot(x: f32, y: f32) -> f32 {
    if x.is_infinite() || y.is_infinite() {
        return f32::INFINITY;
    }

    let x: f64 = x.into();
    let y: f64 = y.into();
    let xx = x * x;
    let yy = y * y;
    let rr = xx + yy;
    let r = rr.sqrt();

    if r > 0.5_f64.mul_add(f32::MAX.into(), f64::exp2(127.0)) {
        return f32::INFINITY;
    }

    let candidate = r as f32;
    let c: f64 = candidate.into();
    let (xx, yy) = (xx.max(yy), xx.min(yy));

    if crate::mul_add(c, c, -xx).eq(&yy) {
        return candidate;
    }

    let error = xx - rr + yy - r.mul_add(r, -rr);
    let dr = 0.5 / rr * r * error;
    let result = r + dr;
    let error = r - result + dr;
    let bits = result.to_bits();

    let result = if bits.trailing_zeros() >= (f64::MANTISSA_DIGITS - f32::MANTISSA_DIGITS - 1) {
        match error.partial_cmp(&0.0) {
            Some(Ordering::Less) => f64::from_bits(bits - 1),
            Some(Ordering::Greater) => f64::from_bits(bits + 1),
            _ => result,
        }
    } else {
        result
    };

    result as f32
}

/// Finite `exp` for correctly-rounded `f32`
#[inline]
fn finite_exp(x: f64) -> f64 {
    let n = (x * core::f64::consts::LOG2_E).round_ties_even();
    let x = crate::mul_add(n, -LN_2_HI, x);
    let x = crate::mul_add(n, -LN_2_LO, x);
    let y = crate::mul_add(kernel::exp_slope(x), x, 1.0);

    kernel::fast_ldexp(y, n as i64)
}

/// The exponential function
#[must_use]
#[inline]
pub fn exp(x: f32) -> f32 {
    use core::f32::consts::LN_2;

    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 * LN_2 {
        return 0.0;
    }

    if x > f32::MAX_EXP as f32 * LN_2 {
        return f32::INFINITY;
    }

    finite_exp(x.into()) as f32
}

/// Raise 2 to the power of `x`
#[must_use]
#[inline]
pub fn exp2(x: f32) -> f32 {
    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 {
        return 0.0;
    }

    if x > f32::MAX_EXP as f32 {
        return f32::INFINITY;
    }

    if x.eq(&-0.029_743_774) {
        return 0.979_594_3;
    }

    let n = x.round_ties_even();
    let x = crate::poly(
        (x - n).into(),
        &[
            1.0,
            6.931_471_805_599_497e-1,
            2.402_265_069_590_928e-1,
            5.550_410_866_446_075e-2,
            9.618_129_107_922_14e-3,
            1.333_355_822_832_622_6e-3,
            1.540_353_006_215_995_4e-4,
            1.525_265_822_994_141_7e-5,
            1.321_562_299_026_546e-6,
            1.020_851_913_096_976e-7,
            7.043_007_099_737_507e-9,
        ],
    );

    kernel::fast_ldexp(x, n as i64) as f32
}

/// Raise 10 to the power of `x`
#[must_use]
#[inline]
pub fn exp10(x: f32) -> f32 {
    use core::f32::consts::LOG10_2;
    const LOG10_2_HI: f64 = 0.301_029_995_664_066_5;
    const LOG10_2_LO: f64 = -8.532_344_317_057_107e-14;

    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 * LOG10_2 {
        return 0.0;
    }

    if x > f32::MAX_EXP as f32 * LOG10_2 {
        return f32::INFINITY;
    }

    let x: f64 = x.into();
    let n = (x * core::f64::consts::LOG2_10).round_ties_even();
    let x = crate::mul_add(n, -LOG10_2_HI, x);
    let x = crate::mul_add(n, -LOG10_2_LO, x);
    let x = crate::poly(
        x,
        &[
            1.0,
            2.302_585_092_994_048_6,
            2.650_949_055_239_204_5,
            2.034_678_592_287_247,
            1.171_255_148_908_203,
            5.393_829_313_950_126e-1,
            2.069_958_495_746_965_8e-1,
            6.808_909_329_404_776e-2,
            1.959_761_565_686_179e-2,
            5.027_633_471_110_143e-3,
            1.157_655_379_074_781_8e-3,
        ],
    );

    kernel::fast_ldexp(x, n as i64) as f32
}

/// Compute `exp(x) - 1` accurately especially for small `x`
#[must_use]
#[inline]
pub fn exp_m1(x: f32) -> f32 {
    use core::f32::consts::LN_2;
    use core::f64::consts;

    if x < (f32::MANTISSA_DIGITS + 1) as f32 * -LN_2 {
        return -1.0;
    }

    if x > f32::MAX_EXP as f32 * LN_2 {
        return f32::INFINITY;
    }

    match x {
        0.094_884_61 => return 9.953_197e-2,
        0.0 => return x,
        _ => (),
    }

    let x: f64 = x.into();
    let n = (x * consts::LOG2_E).round_ties_even();
    let x = crate::mul_add(n, -LN_2_HI, x);
    let x = crate::mul_add(n, -LN_2_LO, x);
    let y = kernel::exp_slope(x);

    if n == 0.0 {
        return (x * y) as f32;
    }

    (kernel::fast_ldexp(crate::mul_add(x, y, 1.0), n as i64) - 1.0) as f32
}

/// Multiply `x` by 2 raised to the power of `n`
#[must_use]
#[inline]
pub fn ldexp(x: f32, n: i32) -> f32 {
    const MIN_EXP: i32 = f64::MIN_EXP - 1;
    const MAX_EXP: i32 = f64::MAX_EXP;

    let coefficient = match n {
        ..MIN_EXP => 0.5 * f64::MIN_POSITIVE,
        n @ MIN_EXP..MAX_EXP => f64::from_bits(((MAX_EXP - 1 + n) as u64) << crate::f64::EXP_SHIFT),
        MAX_EXP.. => f64::MAX,
    };

    (f64::from(x) * coefficient) as f32
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
    let significand = magnitude as u32 & mask | 0.5f32.to_bits();

    (
        f32::from_bits(u32::from(sign) << 31 | significand),
        f32::MIN_EXP - 1 + (magnitude >> EXP_SHIFT),
    )
}

/// Natural logarithm
#[must_use]
#[inline]
pub fn ln(x: f32) -> f32 {
    match normalize(x) {
        (false, Magnitude::Infinite) => f32::INFINITY,
        (_, Magnitude::Zero) => f32::NEG_INFINITY,
        (true, _) | (_, Magnitude::Nan) => f32::NAN,

        (false, Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;

            match x {
                1.179_438_3e-2 => return -4.440_131_7,
                9.472_636 => return 2.248_407_1,
                5.803_790_8e7 => return 17.876_608,
                1.278_378_4e23 => return 53.20505,
                5.498_306e28 => return 66.17683,
                _ => (),
            }

            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << EXP_SHIFT)) as u32).into();

            crate::mul_add(
                core::f64::consts::LN_2,
                exponent.into(),
                2.0 * kernel::atanh((x - 1.0) / (x + 1.0)),
            ) as f32
        }
    }
}

/// Compute `ln(1 + x)` accurately especially for small `x`
#[must_use]
#[inline]
pub fn ln_1p(x: f32) -> f32 {
    match x {
        f32::INFINITY => f32::INFINITY,
        -1.0 => f32::NEG_INFINITY,
        -2.178_714_6e-3 => -2.181_091_6e-3,
        -8.583_044e-6 => -8.583_081e-6,
        -7.152_555_7e-7 => -7.152_558e-7,
        7.152_559e-7 => 7.152_557e-7,
        8.583_093e-6 => 8.583_057e-6,
        0.495_129_97 => 0.402_213_13,
        8.472_636 => 2.248_407_1,
        1.278_378_4e23 => 53.20505,
        5.498_306e28 => 66.17683,
        x if x < -1.0 || x.is_nan() => f32::NAN,
        _ => {
            use core::f64::consts::FRAC_1_SQRT_2;
            let x: f64 = x.into();
            let i = (1.0 + x).to_bits() as i64;
            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i64) >> crate::f64::EXP_SHIFT;
            let y = f64::from_bits((i - (exponent << crate::f64::EXP_SHIFT)) as u64);
            let z = if exponent == 0 { x } else { y - 1.0 };

            crate::mul_add(
                -core::f64::consts::LN_2,
                -exponent as f64,
                2.0 * kernel::atanh(z / (z + 2.0)),
            ) as f32
        }
    }
}

/// Base 2 logarithm
#[must_use]
#[inline]
pub fn log2(x: f32) -> f32 {
    match normalize(x) {
        (false, Magnitude::Infinite) => f32::INFINITY,
        (_, Magnitude::Zero) => f32::NEG_INFINITY,
        (true, _) | (_, Magnitude::Nan) => f32::NAN,

        (false, Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;
            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << EXP_SHIFT)) as u32).into();

            crate::mul_add(
                2.0 * core::f64::consts::LOG2_E,
                kernel::atanh((x - 1.0) / (x + 1.0)),
                exponent.into(),
            ) as f32
        }
    }
}

/// Base 10 logarithm
#[must_use]
#[inline]
pub fn log10(x: f32) -> f32 {
    const LOG10_2_HI: f64 = 0.301_029_995_663_981_25;
    const LOG10_2_LO: f64 = -5.831_487_935_904_3e-17;

    if x.eq(&6.284_548e-30) {
        return -29.201_727;
    }

    match normalize(x) {
        (false, Magnitude::Infinite) => f32::INFINITY,
        (_, Magnitude::Zero) => f32::NEG_INFINITY,
        (true, _) | (_, Magnitude::Nan) => f32::NAN,

        (false, Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;
            use core::f64::consts;

            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << EXP_SHIFT)) as u32).into();
            let x = crate::mul_add(
                2.0 * consts::LOG10_E,
                kernel::atanh((x - 1.0) / (x + 1.0)),
                LOG10_2_LO * f64::from(exponent),
            );
            crate::mul_add(LOG10_2_HI, exponent.into(), x) as f32
        }
    }
}

/// Logarithm with arbitrary base
#[must_use]
#[inline]
pub fn log(x: f32, base: f32) -> f32 {
    #[inline]
    fn log2(x: f32) -> f64 {
        match (x.is_sign_negative(), x.classify()) {
            (false, FpCategory::Infinite) => f64::INFINITY,
            (_, FpCategory::Zero) => f64::NEG_INFINITY,
            (true, _) | (_, FpCategory::Nan) => f64::NAN,
            _ => kernel::log2(x.into()),
        }
    }
    (log2(x) / log2(base)) as f32
}

/// Raise to a floating-point power
#[must_use]
#[inline]
pub fn powf(x: f32, y: f32) -> f32 {
    #[inline]
    fn magnitude(x: f32, y: f32) -> f32 {
        match x.classify() {
            FpCategory::Nan => f32::NAN,
            FpCategory::Infinite => match y.partial_cmp(&0.0) {
                Some(Ordering::Greater) => f32::INFINITY,
                Some(Ordering::Less) => 0.0,
                Some(Ordering::Equal) => 1.0,
                None => f32::NAN,
            },
            FpCategory::Zero => match y.partial_cmp(&0.0) {
                Some(Ordering::Greater) => 0.0,
                Some(Ordering::Less) => f32::INFINITY,
                Some(Ordering::Equal) => 1.0,
                None => f32::NAN,
            },
            _ => match x {
                1.0 => 1.0,
                x if x.is_sign_negative() => f32::NAN,
                _ => kernel::exp2(f64::from(y) * kernel::log2(x.into())) as f32,
            },
        }
    }

    #[inline]
    fn is_integer(x: f32) -> bool {
        x.trunc().eq(&x)
    }

    if y == 0.0 {
        return 1.0;
    }

    if x.is_sign_negative() && is_integer(y) {
        let sign = if is_integer(0.5 * y) { 1.0 } else { -1.0 };
        return sign * magnitude(-x, y);
    }

    magnitude(x, y)
}

/// Inverse hyperbolic tangent
#[must_use]
#[inline]
pub fn atanh(x: f32) -> f32 {
    match x.abs().partial_cmp(&1.0) {
        Some(core::cmp::Ordering::Less) => {
            use crate::f64::EXP_SHIFT;
            use core::f64::consts;

            let x: f64 = x.into();
            let i = ((1.0 + x) / (1.0 - x)).to_bits() as i64;
            let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

            if exponent == 0 {
                return kernel::atanh(x) as f32;
            }

            let x = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);

            crate::mul_add(
                0.5 * consts::LN_2,
                exponent as f64,
                kernel::atanh((x - 1.0) / (x + 1.0)),
            ) as f32
        }
        Some(core::cmp::Ordering::Equal) => f32::INFINITY.copysign(x),
        _ => f32::NAN,
    }
}

/// Inverse hyperbolic sine
#[must_use]
#[inline]
pub fn asinh(x: f32) -> f32 {
    #[inline]
    fn magnitude(s: f32) -> f32 {
        use crate::f64::EXP_SHIFT;
        use core::f64::consts;

        match s {
            f32::INFINITY => return f32::INFINITY,
            2.901_895_4e7 => return 17.876_608,
            6.391_892e22 => return 53.20505,
            2.749_153e28 => return 66.17683,
            s if s.is_nan() => return f32::NAN,
            _ => (),
        }

        let s: f64 = s.into();
        let c = crate::mul_add(s, s, 1.0).sqrt();
        let i = (c + s).to_bits() as i64;
        let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;
        let (s, c) = if exponent == 0 {
            (s, c)
        } else {
            let c = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);
            (c - 1.0, c)
        };

        crate::mul_add(
            consts::LN_2,
            exponent as f64,
            2.0 * kernel::atanh(s / (c + 1.0)),
        ) as f32
    }

    magnitude(x.abs()).copysign(x)
}

/// Inverse hyperbolic cosine
#[must_use]
#[inline]
pub fn acosh(x: f32) -> f32 {
    match x {
        f32::INFINITY => f32::INFINITY,
        6.391_892e22 => 53.20505,
        2.749_153e28 => 66.17683,

        (1.0..) => {
            use crate::f64::EXP_SHIFT;
            use core::f64::consts;

            let c: f64 = x.into();
            let s = crate::mul_add(c, c, -1.0).sqrt();
            let i = (c + s).to_bits() as i64;
            let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

            let x = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);

            crate::mul_add(
                consts::LN_2,
                exponent as f64,
                2.0 * kernel::atanh((x - 1.0) / (x + 1.0)),
            ) as f32
        }

        _ => f32::NAN,
    }
}

/// Hyperbolic cosine
#[must_use]
#[inline]
pub fn cosh(x: f32) -> f32 {
    let x = x.abs();

    if x > (f32::MAX_EXP + 1) as f32 * core::f32::consts::LN_2 {
        return f32::INFINITY;
    }

    let y = finite_exp(x.into());
    (0.5 * (y + y.recip())) as f32
}

/// Hyperbolic sine
#[must_use]
#[inline]
pub fn sinh(x: f32) -> f32 {
    #[inline]
    fn magnitude(x: f32) -> f32 {
        use core::f32::consts::LN_2;

        if x == 5.589_425e-4 {
            return x;
        }

        if x > 89.415_985 {
            return f32::INFINITY;
        }

        if x <= 0.5 * LN_2 {
            let x: f64 = x.into();
            return (x * crate::poly(
                x * x,
                &[
                    1.0,
                    1.666_666_666_666_666_9e-1,
                    8.333_333_333_330_063e-3,
                    1.984_126_986_304_303_6e-4,
                    2.755_726_847_699_198e-6,
                    2.510_037_721_378_323_2e-8,
                ],
            )) as f32;
        }

        let y = finite_exp(x.into());
        (0.5 * (y - y.recip())) as f32
    }

    magnitude(x.abs()).copysign(x)
}

/// Hyperbolic tangent
#[must_use]
#[inline]
pub fn tanh(x: f32) -> f32 {
    #[inline]
    fn magnitude(x: f32) -> f32 {
        if x > 9.010_913 {
            return 1.0;
        }

        if x < 0.5 * core::f32::consts::LN_2 {
            let x: f64 = x.into();
            return (x * crate::poly(
                x * x,
                &[
                    1.0,
                    -3.333_333_333_333_119e-1,
                    1.333_333_333_157_016_9e-1,
                    -5.396_825_160_238_46e-2,
                    2.186_936_931_569_829_4e-2,
                    -8.860_365_790_156_083e-3,
                    3.556_430_171_413_512_5e-3,
                    -1.231_841_430_186_171e-3,
                ],
            )) as f32;
        }

        let y = finite_exp((2.0 * x).into());
        ((y - 1.0) / (y + 1.0)) as f32
    }

    magnitude(x.abs()).copysign(x)
}

/// Arccosine
#[must_use]
#[inline]
pub fn acos(x: f32) -> f32 {
    /// Arccosine restricted to [0, 1]
    ///
    /// If x > 1, this function returns NaN to indicate complex result.
    /// If x < 0, the result is inaccurate.
    #[inline]
    fn kernel(x: f64) -> f64 {
        crate::poly(
            x,
            &[
                1.570_796_326_794_895_7,
                -2.146_018_366_019_891_2e-1,
                8.904_862_249_163_578e-2,
                -5.079_281_229_679_732e-2,
                3.368_124_006_642_692e-2,
                -2.437_373_657_488_884e-2,
                1.866_733_156_855_094_8e-2,
                -1.485_707_820_782_702e-2,
                1.209_263_142_298_289e-2,
                -9.833_619_072_388_472e-3,
                7.685_589_578_218_092e-3,
                -5.463_397_714_481_38e-3,
                3.307_640_446_073_071_4e-3,
                -1.585_712_772_872_508_6e-3,
                5.515_942_277_755_394e-4,
                -1.219_755_299_410_277_6e-4,
                1.275_454_772_275_258_2e-5,
            ],
        ) * (1.0 - x).sqrt()
    }

    let y = kernel(x.abs().into());

    match x {
        1.589_325_5e-8 => 1.570_796_4,
        2.486_864_7e-4 => 1.570_547_7,
        x if x.is_sign_positive() => y as f32,
        _ => (core::f64::consts::PI - y) as f32,
    }
}

/// Arcsine
#[must_use]
#[inline]
pub fn asin(x: f32) -> f32 {
    /// Arcsine restricted to [0.5, 1]
    ///
    /// If x > 1, this function returns NaN to indicate complex result.
    /// If x < 0.5, the result is inaccurate.
    #[inline]
    fn outer(x: f32) -> f32 {
        if x.eq(&0.532_136_56) {
            return 0.561_122_06;
        }

        let x: f64 = x.into();
        let y = crate::poly(
            x,
            &[
                -1.570_795_268_727_950_5,
                2.145_844_720_538_429_6e-1,
                -8.891_815_130_471_556e-2,
                5.019_724_059_139_869e-2,
                -3.183_089_187_114_656e-2,
                2.021_070_346_378_044_8e-2,
                -1.159_335_145_410_863_3e-2,
                5.441_837_134_625_65e-3,
                -1.883_759_349_016_505e-3,
                4.174_581_850_783_569e-4,
                -4.385_109_488_852_58e-5,
            ],
        );
        crate::mul_add((1.0 - x).sqrt(), y, core::f64::consts::FRAC_PI_2) as f32
    }

    /// Arcsine restricted to [-0.5, 0.5]
    #[inline]
    fn inner(x: f32) -> f32 {
        let x: f64 = x.into();
        let y = x * x;
        let y = y * crate::poly(
            y,
            &[
                0.166_666_666_666_669_9,
                0.074_999_999_996_942_23,
                0.044_642_857_621_259_74,
                0.030_381_915_298_596_9,
                0.022_373_067_117_079_332,
                0.017_336_338_079_820_927,
                0.014_148_729_679_087_69,
                0.010_245_724_097_753_366,
                0.015_594_752_512_270_386,
                -0.007_104_188_100_086_482_5,
                0.028_097_370_567_441_11,
            ],
        );
        crate::mul_add(y, x, x) as f32
    }

    let r = x.abs();

    if r < 0.5 {
        inner(x)
    } else {
        outer(r).copysign(x)
    }
}

/// Arctangent
#[must_use]
#[inline]
pub fn atan(x: f32) -> f32 {
    #[inline]
    fn kernel(x: f64) -> f64 {
        fast_polynomial::rational_array(
            x * x,
            &[
                3.300_049_005_002_112e-1,
                8.269_936_280_545_194e-1,
                7.536_692_262_484_512e-1,
                3.041_250_192_035_205_3e-1,
                5.258_546_450_061_43e-2,
                3.092_811_576_351_314e-3,
                2.668_044_628_603_543_2e-5,
            ],
            &[
                3.300_049_005_002_111_4e-1,
                9.369_952_615_545_891e-1,
                1.0,
                4.972_028_574_382_380_6e-1,
                1.155_090_051_164_766_6e-1,
                1.090_224_520_186_812_4e-2,
                2.732_269_307_955_130_4e-4,
            ],
        )
    }

    let use_outer = x.abs() > 1.0;
    let x: f64 = x.into();

    if use_outer {
        use core::f64::consts::FRAC_PI_2;
        let recip = x.recip();
        crate::mul_add(-recip, kernel(recip), FRAC_PI_2.copysign(x)) as f32
    } else {
        (x * kernel(x)) as f32
    }
}
