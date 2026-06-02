#![allow(clippy::pedantic, clippy::approx_constant)]
#![warn(clippy::unreadable_literal)]

mod kernel;
use crate::f64::kernel::Sum;
use crate::Sign;
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
    let y: f64 = f32::from_bits(crate::u32_sign_bit(sign) | magnitude).into();
    let y = y * (0.5 + 1.5 * x / crate::mul_add(2.0 * y, y * y, x));
    let y = y * (0.5 + 1.5 * x / crate::mul_add(2.0 * y, y * y, x));

    y as f32
}

/// Hypotenuse of a right-angled triangle with sides `x` and `y`
///
/// Mirrors CORE-MATH's hot path: sort to `at ≥ ay`, compute `r² = at² + ay²`
/// and `r = √r²` in f64, then dispatch on cheap integer-bit tests.  The
/// degenerate `ay/at < 2⁻¹³` case skips the square root entirely; the bulk of
/// `r` values are far from an f32 midpoint and return the single rounding `c =
/// r as f32`; only the tight-rounding sliver runs the residual correction.
#[must_use]
#[inline]
pub fn hypot(x: f32, y: f32) -> f32 {
    /// Largest f64 ≤ 2⁻¹³ (= `0x1.fffffep-13`); the small-`ay` cutoff
    const SMALL_RATIO: f64 = f64::from_bits(0x3F2F_FFFF_C000_0000);

    /// 2⁻¹³ as f32; the FMA coefficient on the small-`ay` fast path
    const HALF_ULP_F32: f32 = f32::from_bits(0x3900_0000);

    /// `f32::MAX` cast to f64 (`0x1.fffffep+127`); above this, `r as f32` overflows
    const F32_MAX_AS_F64: u64 = 0x47EF_FFFF_E000_0000;

    /// Width of the f64 → f32 rounding window: 28 = 52 − 24 low bits
    const TAIL_MASK: u64 = 0x0FFF_FFFF;

    let ax = x.abs();
    let ay = y.abs();

    if ax == f32::INFINITY || ay == f32::INFINITY {
        return f32::INFINITY;
    }
    if !ax.is_finite() || !ay.is_finite() {
        return ax + ay; // propagates NaN
    }

    let big_f32 = ax.max(ay);
    let small_f32 = ax.min(ay);
    let big = f64::from(big_f32);
    let small = f64::from(small_f32);
    let big2 = big * big;
    let small2 = small * small;

    // `small ≤ 2⁻¹³·big` ⇒ hypot ≈ big + small²/(2·big), and `big + 2⁻¹³·small`
    // via a single FMA correctly rounds throughout that range without ever
    // taking a sqrt.
    if small < big * SMALL_RATIO {
        return HALF_ULP_F32.mul_add(small_f32, big_f32);
    }

    let r2 = big2 + small2;
    let r = r2.sqrt();
    let candidate = r as f32;

    // Past f32::MAX, the single rounding `r as f32` already produces the
    // correct overflow (MAX or +∞); the midpoint gate below would read
    // meaningless low bits.
    if r.to_bits() > F32_MAX_AS_F64 {
        return candidate;
    }

    // Common case: `r`'s low f64 tail is far from an f32 midpoint, so the
    // single rounding is correct.
    if (r.to_bits().wrapping_add(1) & TAIL_MASK) > 2 {
        return candidate;
    }

    // Residual test: if `candidate² == big² + small²` exactly, we're done.
    let candidate_f64 = f64::from(candidate);
    if crate::mul_add(candidate_f64, candidate_f64, -big2) - small2 == 0.0 {
        return candidate;
    }

    // One-step Newton correction in f64, then round-to-odd via the sign of the
    // low part of the (r_hi, r_lo) double-double.
    let residual = (big2 - r2) + small2 - r.mul_add(r, -r2);
    let correction = r * (0.5 / r2) * residual;
    let r_hi = r + correction;
    let r_lo = correction + (r - r_hi);
    let bits = r_hi.to_bits();

    let bits = match (bits & TAIL_MASK, r_lo.partial_cmp(&0.0)) {
        (0, Some(Ordering::Greater)) => bits.wrapping_add(1),
        (0, Some(Ordering::Less)) => bits.wrapping_sub(1),
        _ => bits,
    };
    f64::from_bits(bits) as f32
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
///
/// Uses the identity `exp(x) − 1 = 2ⁿ·(exp(r)−1) + (2ⁿ−1)` to avoid
/// catastrophic cancellation when `2ⁿ·exp(r) ≈ 1` and to unify the `n = 0`
/// near-zero case with the main path.
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

    if x == 0.0 {
        return x;
    }

    let x: f64 = x.into();
    let n = (x * consts::LOG2_E).round_ties_even();
    let r = crate::mul_add(n, -LN_2_HI, x);
    let r = crate::mul_add(n, -LN_2_LO, r);
    let y = kernel::exp_slope(r);

    (kernel::fast_ldexp(r * y, n as i64) + (crate::exp2i(n as i64) - 1.0)) as f32
}

/// Multiply `x` by 2 raised to the power of `n`
#[must_use]
#[inline]
pub const fn ldexp(x: f32, n: i32) -> f32 {
    const MIN_EXP: i32 = f64::MIN_EXP - 1;
    const MAX_EXP: i32 = f64::MAX_EXP;

    let coefficient = match n {
        ..MIN_EXP => 0.5 * f64::MIN_POSITIVE,
        n @ MIN_EXP..MAX_EXP => f64::from_bits(((MAX_EXP - 1 + n) as u64) << crate::f64::EXP_SHIFT),
        MAX_EXP.. => f64::MAX,
    };

    (x as f64 * coefficient) as f32
}

/// Decompose into a significand and an exponent
///
/// The absolute value of the significand is in the range of [0.5, 1) for
/// nonzero finite `x` for historical reasons.  This function also explains how
/// [`f32::MAX_EXP`] and [`f32::MIN_EXP`] are defined.
#[must_use]
#[inline]
pub const fn frexp(x: f32) -> (f32, i32) {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return (x, 0);
    };

    let mask = f32::MIN_POSITIVE.to_bits() - 1;
    let significand = magnitude as u32 & mask | 0.5f32.to_bits();

    (
        f32::from_bits(crate::u32_sign_bit(sign) | significand),
        f32::MIN_EXP - 1 + (magnitude >> EXP_SHIFT),
    )
}

/// Natural logarithm
#[must_use]
#[inline]
pub fn ln(x: f32) -> f32 {
    match normalize(x) {
        (Sign::Positive, Magnitude::Infinite) => f32::INFINITY,
        (_, Magnitude::Zero) => f32::NEG_INFINITY,
        (Sign::Negative, _) | (_, Magnitude::Nan) => f32::NAN,

        (Sign::Positive, Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;

            // TODO: these hard-coded returns are correctly-rounded values for
            // double-rounding cases (the exact ln is within ~½ f64-ulp of an f32
            // midpoint, so the f64 kernel rounds the wrong way).  Remove them by
            // rounding the f64 result to odd / carrying a hi+lo pair before the
            // final f32 round, as the f64 logarithm functions now do.
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
    // TODO: the hard-coded returns below are double-rounding cases (see the note
    // in `ln`); remove them with a round-to-odd / hi+lo final round.
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
        (Sign::Positive, Magnitude::Infinite) => f32::INFINITY,
        (_, Magnitude::Zero) => f32::NEG_INFINITY,
        (Sign::Negative, _) | (_, Magnitude::Nan) => f32::NAN,

        (Sign::Positive, Magnitude::Normalized(i)) => {
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
        (Sign::Positive, Magnitude::Infinite) => f32::INFINITY,
        (_, Magnitude::Zero) => f32::NEG_INFINITY,
        (Sign::Negative, _) | (_, Magnitude::Nan) => f32::NAN,

        (Sign::Positive, Magnitude::Normalized(i)) => {
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

#[cfg(feature = "core-math")]
pub use core_math::powf;

/// Raise to a floating-point power
#[cfg(not(feature = "core-math"))]
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
    use crate::f64::EXP_SHIFT;
    use core::f64::consts;
    let s = x.abs();

    let magnitude = match s {
        f32::INFINITY => f32::INFINITY,
        2.901_895_4e7 => 17.876_608,
        6.391_892e22 => 53.20505,
        2.749_153e28 => 66.17683,
        s if s.is_nan() => f32::NAN,
        _ => {
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
    };

    magnitude.copysign(x)
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

/// `sinh(r) / r = P(r²)` on `r ∈ [-½·ln 2, ½·ln 2]`, relative error ≈ 2⁻⁴³
///
/// `ratapprox --function="sinh(x)/x" --dom="[0.001,0.347]"
///   --num="[1,x^2,x^4,x^6,x^8,x^10]" --den="[1]"`
const SINH_CORE: [f64; 6] = [
    1.0,
    1.666_666_666_666_666_9e-1,
    8.333_333_333_330_063e-3,
    1.984_126_986_304_303_6e-4,
    2.755_726_847_699_198e-6,
    2.510_037_721_378_323_2e-8,
];

/// `cosh(r) = Q(r²)` on `r ∈ [-½·ln 2, ½·ln 2]`, relative error ≈ 2⁻⁴³
///
/// `ratapprox --function="cosh(x)" --dom="[0.001,0.347]"
///   --num="[1,x^2,x^4,x^6,x^8,x^10]" --den="[1]"`
const COSH_CORE: [f64; 6] = [
    1.0,
    5.0e-1,
    4.166_666_666_651_527e-2,
    1.388_888_894_704_863e-3,
    2.480_148_989_146_468e-5,
    2.763_130_793_803_956e-7,
];

/// Hyperbolic sine
///
/// Uses the addition formula `sinh(n·ln2 + r) = cosh(n·ln2)·sinh(r) +
/// sinh(n·ln2)·cosh(r)` with `sinh(n·ln2) = (2ⁿ − 2⁻ⁿ)/2` and
/// `cosh(n·ln2) = (2ⁿ + 2⁻ⁿ)/2`.  Both half-power values come from
/// `fast_ldexp`, so no division is needed anywhere on the main path.
#[must_use]
#[inline]
pub fn sinh(x: f32) -> f32 {
    let magnitude = match x.abs() {
        5.589_425e-4 => 5.589_425e-4,
        x if x > 89.415_985 => f32::INFINITY,

        x => {
            let x: f64 = x.into();
            let n = (x * core::f64::consts::LOG2_E).round_ties_even();
            let r = crate::mul_add(n, -LN_2_HI, x);
            let r = crate::mul_add(n, -LN_2_LO, r);
            let r2 = r * r;

            let sinh_r = r * crate::poly(r2, &SINH_CORE);
            let cosh_r = crate::poly(r2, &COSH_CORE);

            let n = n as i64;
            let pow_n = kernel::fast_ldexp(1.0, n);
            let pow_neg_n = kernel::fast_ldexp(1.0, -n);
            let sinh_n = 0.5 * (pow_n - pow_neg_n);
            let cosh_n = 0.5 * (pow_n + pow_neg_n);

            crate::mul_add(cosh_n, sinh_r, sinh_n * cosh_r) as f32
        }
    };

    magnitude.copysign(x)
}

/// Hyperbolic tangent
#[must_use]
#[inline]
pub fn tanh(x: f32) -> f32 {
    let magnitude = match x.abs() {
        x if x > 9.010_913 => 1.0,

        x if x < 0.5 * core::f32::consts::LN_2 => {
            let x: f64 = x.into();

            (x * crate::poly(
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
            )) as f32
        }
        x => {
            let y = finite_exp((2.0 * x).into());
            ((y - 1.0) / (y + 1.0)) as f32
        }
    };

    magnitude.copysign(x)
}

/// Minimax coefficients of `P` in `asin(t) = t + t³·P(t²)` on `t² ∈ [0, ¼]`.
///
/// Shared by the `|x| < ½` branch of [`asin`] and by both branches of [`acos`].
/// For `|x| ≥ ½`, reflecting through `s = √((1−|x|)/2) ∈ [0, ½]` keeps the
/// kernel argument in `[0, ¼]`, so the same `P` serves the whole domain:
/// `asin(x) = π/2 − 2·asin(s)` and `acos(x) = 2·asin(s)` (or `π − 2·asin(s)`).
///
/// Relative error ≈ 2⁻⁴⁶, generated with
/// `ratapprox --function="(asin(sqrt(x))-sqrt(x))/(x*sqrt(x))"
///   --dom="[1e-30,0.25]" --num=[1,x,x^2,...,x^10] --den=[1]`.
const ASIN_NEAR_ZERO: [f64; 11] = [
    0.166_666_666_666_669_68,
    0.074_999_999_997_106_53,
    0.044_642_857_599_726_166,
    0.030_381_916_397_495_97,
    0.022_373_038_603_277_58,
    0.017_336_763_281_738_436,
    0.014_144_868_382_976_405,
    0.010_267_430_424_776_843,
    0.015_521_003_704_904_501,
    -0.006_965_450_747_007_442,
    0.027_986_438_522_030_702,
];

/// `asin(t)` for `t ∈ [0, ½]`, where `u = t²`, via `t + t³·P(t²)`.
#[inline]
fn asin_near_zero(t: f64, u: f64) -> f64 {
    crate::mul_add(t * u, crate::poly(u, &ASIN_NEAR_ZERO), t)
}

/// Arccosine
///
/// `acos(x) = π/2 − asin(x)`.  For `|x| < ½` this subtracts the near-zero
/// kernel [`asin_near_zero`] directly (no square root); for `|x| ≥ ½` it
/// reflects through `s = √((1−|x|)/2) ∈ [0, ½]`, giving `acos(x) = 2·asin(s)`
/// for `x ≥ 0` and `π − 2·asin(s)` for `x < 0`.  Both forms are evaluated and
/// selected branchlessly so random inputs pay no misprediction penalty.
#[must_use]
#[inline]
pub fn acos(x: f32) -> f32 {
    let xf: f64 = x.into();
    let a = xf.abs();

    // |x| < ½:  π/2 − asin(x)
    let near = core::f64::consts::FRAC_PI_2 - asin_near_zero(xf, xf * xf);

    // |x| ≥ ½:  2·asin(s) folded about π for x < 0, s = √((1−|x|)/2)
    let u = 0.5 * (1.0 - a);
    let two = 2.0 * asin_near_zero(u.sqrt(), u);
    let far = if xf.is_sign_positive() {
        two
    } else {
        core::f64::consts::PI - two
    };

    let y = (if a < 0.5 { near } else { far }) as f32;

    // Hard-to-round cases near π/2 (tiny x, near-zero branch).
    match x {
        1.589_325_5e-8 => 1.570_796_4,
        2.486_864_7e-4 => 1.570_547_7,
        _ => y,
    }
}

/// Arcsine
///
/// For `|x| < ½`, evaluate the near-zero kernel [`asin_near_zero`] directly.
/// For `|x| ≥ ½`, reflect through `s = √((1−|x|)/2) ∈ [0, ½]` with
/// `asin(x) = π/2 − 2·asin(s)` (sign restored afterward), reusing the same
/// kernel.  Both forms are evaluated and selected branchlessly.
#[must_use]
#[inline]
pub fn asin(x: f32) -> f32 {
    let xf: f64 = x.into();
    let a = xf.abs();

    // |x| < ½:  asin(x) directly
    let near = asin_near_zero(xf, xf * xf);

    // |x| ≥ ½:  ±(π/2 − 2·asin(s)), s = √((1−|x|)/2)
    let u = 0.5 * (1.0 - a);
    let two = 2.0 * asin_near_zero(u.sqrt(), u);
    let far = if xf.is_sign_positive() {
        core::f64::consts::FRAC_PI_2 - two
    } else {
        two - core::f64::consts::FRAC_PI_2
    };

    let y = (if a < 0.5 { near } else { far }) as f32;

    // Hard-to-round case in the reflection branch.
    match x.abs() {
        0.532_136_56 => 0.561_122_06_f32.copysign(x),
        _ => y,
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

/// Arctangent of `y / x` using the signs of both to select the quadrant
///
/// The result is the angle in `(-π, π]` between the positive `x`-axis and the
/// point `(x, y)`, carrying the sign of `y`.
#[must_use]
#[inline]
pub fn atan2(y: f32, x: f32) -> f32 {
    use core::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    /// Correctly-rounded `f32` of 3π/4
    const THREE_PI_4: f32 = 2.356_194_5;

    if x.is_nan() || y.is_nan() {
        return f32::NAN;
    }

    if x.is_infinite() || y.is_infinite() {
        let magnitude = match (x.is_infinite(), y.is_infinite()) {
            (true, true) if x.is_sign_positive() => FRAC_PI_4,
            (true, true) => THREE_PI_4,
            (true, false) if x.is_sign_positive() => 0.0,
            (true, false) => PI,
            (false, true) => FRAC_PI_2,
            (false, false) => unreachable!(),
        };
        return magnitude.copysign(y);
    }

    if y == 0.0 {
        return if x.is_sign_positive() { 0.0 } else { PI }.copysign(y);
    }

    if x == 0.0 {
        return FRAC_PI_2.copysign(y);
    }

    kernel::atan2(x.abs().into(), y.abs().into(), x.is_sign_negative()).copysign(y)
}

/// `erf(x)` for `|x| < 0.4375` as `x·P(x²)`
///
/// The minimax coefficients (from CORE-MATH) and this evaluation order are
/// chosen so that the single `f64` product rounds correctly to `f32`; `c[0]` is
/// the correctly-rounded `2/√π`, keeping the tiny-argument regime exact.
#[inline]
fn erf_small(x: f64) -> f64 {
    const C: [f64; 8] = [
        1.128_379_167_095_512_6,
        -0.376_126_389_031_781_8,
        0.112_837_916_703_424_2,
        -0.026_866_170_388_309_935,
        0.005_223_972_335_150_932_5,
        -0.000_854_773_440_605_154_9,
        0.000_120_184_475_094_822_11,
        -1.372_114_526_702_553_9e-5,
    ];

    let z2 = x * x;
    let z4 = z2 * z2;
    let z8 = z4 * z4;
    let c0 = z4.mul_add(z2.mul_add(C[3], C[2]), z2.mul_add(C[1], C[0]));
    let c4 = z4.mul_add(z2.mul_add(C[7], C[6]), z2.mul_add(C[5], C[4]));

    x * z8.mul_add(c4, c0)
}

/// `erfc(x)` as a double-double for finite `x` in `[0.4375, 0x1.41bbf8p+3]`
///
/// Uses `erfc(x) = (2/(2+x))·exp(Q(t) - x²)` with `t = 2/(2+x)`, where the
/// degree-22 polynomial `Q(t)` approximates `x² + ln(erfc(x)·(2+x)/2)` — a
/// smooth, `O(1)` function.  `x²` is formed exactly and the exponent is carried
/// in double-double so the single-rounding error of `exp` dominates.
///
/// `Q(t)` was generated with
/// `ratapprox --function="(2/x-2)^2 + log(erfc(2/x-2)) - log(x)"
///  --dom="[0.16,0.8205]" --num="[1,x,...,x^22]" --den="[1]" --weight=1`.
fn erfc_dd(x: f64) -> Sum {
    const Q: [f64; 23] = [
        -1.265_512_122_089_828_2,
        0.999_999_909_841_198_7,
        0.375_002_743_471_048_67,
        0.083_281_051_140_198_57,
        -0.085_237_288_049_587_05,
        -0.150_760_071_598_243_3,
        -0.037_328_341_350_081_02,
        -0.307_000_710_902_394_2,
        1.813_760_492_069_090_2,
        -6.732_827_419_146_119,
        22.932_196_277_953_373,
        -63.486_057_493_716_345,
        143.005_106_419_187_14,
        -264.328_775_383_285_2,
        394.457_961_771_325_7,
        -465.717_326_957_287_7,
        428.276_437_921_645_1,
        -302.337_694_534_648_9,
        160.625_086_292_286_43,
        -62.167_646_608_866_96,
        16.552_127_045_415_844,
        -2.710_346_863_587_057_3,
        0.205_553_868_706_903_5,
    ];

    let t = Sum::from_quotient(2.0, 2.0 + x);
    let q = crate::poly(t.high, &Q);

    // exponent W = Q(t) - x², with x² exact and the difference in double-double
    let x2 = Sum::from_product(x, x);
    let w = Sum { high: q, low: 0.0 }
        + Sum {
            high: -x2.high,
            low: -x2.low,
        };

    // exp(W) = exp(W.high)·exp(W.low) ≈ exp(W.high)·(1 + W.low)
    t * (finite_exp(w.high) * (1.0 + w.low))
}

/// The error function
#[must_use]
#[inline]
pub fn erf(x: f32) -> f32 {
    if x.is_nan() {
        return x;
    }

    let ax = x.abs();

    if ax < 0.437_5 {
        return erf_small(x.into()) as f32;
    }

    // erf rounds to ±1 for |x| > 0x1.f5a888p+1 (covers ∞)
    let magnitude = if ax > f32::from_bits(0x407a_d444) {
        1.0
    } else {
        let e = erfc_dd(ax.into());
        kernel::round(
            Sum {
                high: 1.0,
                low: 0.0,
            } + Sum {
                high: -e.high,
                low: -e.low,
            },
        )
    };

    magnitude.copysign(x)
}

/// The complementary error function `1 - erf(x)`
#[must_use]
#[inline]
pub fn erfc(x: f32) -> f32 {
    if x.is_nan() {
        return x;
    }

    let ax = x.abs();

    if ax < 0.437_5 {
        // Lone hard-to-round case in this regime (found by exhaustive sweep):
        // erfc(-0x1.d93ec4p-17) sits a hair past an f32 midpoint.
        if x.to_bits() == 0xb76c_9f62 {
            return f32::from_bits(0x3f80_0085);
        }
        return (1.0 - erf_small(x.into())) as f32;
    }

    // erfc underflows to 0 for large x; the reflection 2 - erfc tends to 2
    if ax >= f32::from_bits(0x4120_ddfc) {
        return if x.is_sign_positive() { 0.0 } else { 2.0 };
    }

    let e = erfc_dd(ax.into());

    if x.is_sign_positive() {
        kernel::round(e)
    } else {
        kernel::round(
            Sum {
                high: 2.0,
                low: 0.0,
            } + Sum {
                high: -e.high,
                low: -e.low,
            },
        )
    }
}

/// Negate a double-double
#[inline]
fn neg(a: Sum) -> Sum {
    Sum {
        high: -a.high,
        low: -a.low,
    }
}

/// Natural logarithm of a positive double-double
///
/// `ln(hi + lo) = ln(hi) + ln(1 + lo/hi) ≈ ln(hi) + lo/hi`, the last term being
/// a tiny correction folded into the double-double `ln`.
#[inline]
fn ln_sum(x: Sum) -> Sum {
    crate::f64::ln_dd(x.high)
        + Sum {
            high: x.low / x.high,
            low: 0.0,
        }
}

/// Round `value · 2`<sup>`q`</sup> to the nearest `f32`, negating when `negative`
///
/// Covers tgamma's full range: overflow to `±∞` for large `q`, gradual
/// underflow to `±0` for very negative `q`, and [`kernel::round_general`] for the
/// normal and subnormal grids in between.
#[inline]
fn finish(value: Sum, q: i64, negative: bool) -> f32 {
    let magnitude = if q >= 1022 {
        f32::INFINITY
    } else if q <= -203 {
        // `value.high · 2^q` underflows even f64 (minimum f64 ≈ 2^-1074); flush to 0.
        0.0
    } else {
        // Renormalize so `high` is the nearest `f64`: the upstream `Sum / z` and
        // `Sum * Sum` can leave the pair denormal by up to an ulp, which would
        // defeat `round`'s round-to-odd at the hardest f32 boundaries.
        kernel::round_general(Sum::from_sum(
            kernel::fast_ldexp(value.high, q),
            kernel::fast_ldexp(value.low, q),
        ))
    };

    if negative {
        -magnitude
    } else {
        magnitude
    }
}

/// Fast plain-`f64` `Γ(z)` over the recurrence range, with a relative error bound
///
/// Mirrors [`tgamma_dd`] in `f64`: reduce `z` into `[2.375, 3.375]`, evaluate the
/// degree-11 minimax, and walk back by the same recurrence.  The error is
/// dominated by the polynomial's `2⁻⁴²`, which the gate in [`tgamma`] uses.
#[inline]
fn tgamma_f64(x: f64) -> (f64, f64) {
    let m = x - kernel::TGAMMA_CENTER;
    let i = m.round_ties_even();
    let mut value = crate::poly(m - i, &kernel::TGAMMA_POLY_F64);
    let steps = i.abs() as i32;

    if i > 0.0 {
        let mut factor = x;
        for _ in 0..steps {
            factor -= 1.0;
            value *= factor;
        }
    } else if i < 0.0 {
        let mut product = x;
        let mut factor = x;
        for _ in 1..steps {
            factor += 1.0;
            product *= factor;
        }
        value /= product;
    }

    (value, crate::exp2i(-37) * value.abs())
}

/// Double-double `Γ(z)` over the recurrence range, the accurate Ziv fallback
///
/// Reduces `z` into the minimax interval `[2.375, 3.375]` centred on 2.875, then
/// walks back with `Γ(z) = Γ(z−i)·∏(z−j)` for `z` above the interval, or a single
/// reciprocal of `∏(z+j)` below it.  For negative `z` the product runs through
/// negative factors, so it supplies the sign as well — no reflection needed.
#[cold]
#[inline(never)]
fn tgamma_dd(x: f64) -> f32 {
    let m = x - kernel::TGAMMA_CENTER;
    let i = m.round_ties_even();
    let mut value = kernel::tgamma_poly(m - i);
    let steps = i.abs() as i32;

    if i > 0.0 {
        let mut factor = x;
        for _ in 0..steps {
            factor -= 1.0;
            value = value * factor;
        }
    } else if i < 0.0 {
        let mut product = Sum { high: x, low: 0.0 };
        let mut factor = x;
        for _ in 1..steps {
            factor += 1.0;
            product = product * factor;
        }
        value = value * product.recip();
    }

    let negative = value.high < 0.0;
    finish(if negative { neg(value) } else { value }, 0, negative)
}

/// The gamma function
#[must_use]
#[inline]
pub fn tgamma(z: f32) -> f32 {
    if z.is_nan() {
        return z;
    }

    if z == f32::INFINITY {
        return f32::INFINITY;
    }

    if z == 0.0 {
        return f32::INFINITY.copysign(z);
    }

    let x = f64::from(z);

    // Near the pole at 0, Γ(z) ≈ 1/z.  The Maclaurin series Γ(z) = 1/z − γ + c₂z +
    // c₃z² + c₄z³ keeps the dynamic range inside the double-double 1/z while the
    // O(1) correction stays ample in f64; this also yields Γ(±0) = ±∞.
    if z.abs() < 0.000_244_140_625 {
        let correction = crate::poly(
            x,
            &[
                -0.577_215_664_901_532_9,
                0.989_055_995_327_972_6,
                -0.907_479_076_080_886_3,
                0.981_728_086_834_400_2,
            ],
        );
        let value = Sum::from_quotient(1.0, x)
            + Sum {
                high: correction,
                low: 0.0,
            };
        return kernel::round_signed(value);
    }

    // Γ exceeds f32::MAX at z ≈ 35.0401; bail before the recurrence loop, which
    // would otherwise run unboundedly for huge z.
    if z >= 35.040_100_097_656_25 {
        return f32::INFINITY;
    }

    // Non-positive integers are poles (z = 0 handled above); below −42 the
    // magnitude underflows past 2⁻¹⁵¹ to a signed zero alternating with each pole.
    if x == x.floor() && z < 0.0 {
        return f32::NAN;
    }
    if z < -42.0 {
        return if (x.floor() as i64) & 1 == 0 {
            0.0
        } else {
            -0.0
        };
    }

    // Ziv two-step over the recurrence range: the plain-f64 path is correctly
    // rounded unless its value lands within the error bound of an f32 boundary,
    // where the double-double path resolves it.
    let (value, err) = tgamma_f64(x);
    let lo = (value - err) as f32;
    let hi = (value + err) as f32;

    if lo == hi {
        lo
    } else {
        tgamma_dd(x)
    }
}

/// `ln Γ(y)` as a double-double for `y ≥ ½`
///
/// Reduces `y` upward to `t ≥ 14` with `ln Γ(y) = ln Γ(t) − ln ∏(y+j)`, then
/// applies the Stirling expansion `(t−½)·ln t − t + ½ln(2π) + 1/(12t) +
/// tail(1/t²)`.  The big `(t−½)·ln t − t` and the leading `1/(12t)` stay in
/// double-double; the asymptotic tail is negligible in f64.  Relative error
/// near `2⁻⁶⁴`.
#[inline]
fn lgamma_pos_dd(y: f64) -> Sum {
    let mut product = Sum {
        high: 1.0,
        low: 0.0,
    };
    let mut t = y;
    let mut reduced = false;

    while t < kernel::LGAMMA_STIRLING {
        product = product * t;
        t += 1.0;
        reduced = true;
    }

    let u = 1.0 / (t * t);
    let rest = crate::poly(u, &kernel::LGAMMA_TAIL) * (u / t);

    let stirling = crate::f64::ln_dd(t) * (t - 0.5)
        + Sum { high: -t, low: 0.0 }
        + kernel::HALF_LN_2PI
        + Sum::from_quotient(1.0, 12.0 * t)
        + Sum {
            high: rest,
            low: 0.0,
        };

    if reduced {
        stirling + neg(ln_sum(product))
    } else {
        stirling
    }
}

/// `ln Γ(y)` in plain `f64` for `y ≥ ½`, the fast Ziv path
///
/// Below 8 a log-free rational `(y−1)(y−2)·g(y)` does the work; from 8 up the
/// Stirling series with a single logarithm takes over (its `2⁻⁴⁶` tail
/// truncation sits well inside the [`lgamma`] gate).
#[inline]
fn lgamma_pos_f64(y: f64) -> f64 {
    if y < 8.0 {
        let g = crate::poly(y, &kernel::LGAMMA_NUM) / crate::poly(y, &kernel::LGAMMA_DEN);
        return (y - 1.0) * (y - 2.0) * g;
    }

    let u = 1.0 / (y * y);
    let s = 1.0 / (12.0 * y) + crate::poly(u, &kernel::LGAMMA_TAIL) * (u / y);
    (y - 0.5) * y.ln() - y + 0.918_938_533_204_672_8 + s
}

/// Fast `f64` approximation of `ln|Γ(z)|` with an absolute error bound
///
/// The fast path's relative error (`≈2⁻³⁷` from the rational, less from Stirling)
/// turns into an *absolute* error proportional to the magnitude actually fed
/// through it — `ln Γ(1−z)` under reflection, where the reflected result itself
/// can be tiny.  Returning the bound lets [`lgamma`] gate tightly instead of
/// assuming the worst case everywhere.
#[inline]
fn lgamma_f64(z: f32) -> (f64, f64) {
    /// `ln(π)`, the reflection constant (kept literal — `f64::ln` is not `const`)
    const LN_PI: f64 = 1.144_729_885_849_400_2;

    let x = f64::from(z);
    if z < 0.5 {
        let reflected = lgamma_pos_f64(1.0 - x);
        let value = LN_PI - kernel::sinpi(z).abs().ln() - reflected;
        (
            value,
            crate::exp2i(-36) * reflected.abs() + crate::exp2i(-44),
        )
    } else {
        let value = lgamma_pos_f64(x);
        (value, crate::exp2i(-36) * value.abs() + crate::exp2i(-44))
    }
}

/// The double-double `ln|Γ(z)|`, the accurate Ziv fallback for [`lgamma`]
///
/// The caller has already handled NaN, ±∞, the non-positive-integer poles and
/// the exact zeros `Γ(1) = Γ(2) = 1`.  Kept out of line so the rare fallback
/// never bloats the hot path.
#[cold]
#[inline(never)]
fn lgamma_dd(z: f32) -> f32 {
    if z < 0.5 {
        // ln|Γ(z)| = ln π − ln|sin(πz)| − ln Γ(1−z), the reflection formula.
        let value = ln_sum(kernel::PI)
            + neg(crate::f64::ln_dd(kernel::sinpi(z).abs()))
            + neg(lgamma_pos_dd(1.0 - f64::from(z)));
        return kernel::round_signed(value);
    }

    kernel::round_signed(lgamma_pos_dd(f64::from(z)))
}

/// The natural logarithm of the absolute value of the gamma function
#[must_use]
#[inline]
pub fn lgamma(z: f32) -> f32 {
    if z == 0.0 || z == f32::INFINITY {
        return f32::INFINITY;
    }

    if z.is_nan() {
        return z;
    }

    // Non-positive integers (and −∞) are poles; Γ(1) = Γ(2) = 1 give exact zeros.
    if z < 0.5 && z.round_ties_even() == z {
        return f32::INFINITY;
    }
    if z == 1.0 || z == 2.0 {
        return 0.0;
    }

    // Ziv two-step: the plain-f64 path is correctly rounded unless the value
    // lands within `err` of an f32 boundary, where the double-double path
    // resolves it.
    let (f, err) = lgamma_f64(z);
    let lo = (f - err) as f32;
    let hi = (f + err) as f32;

    if lo == hi {
        lo
    } else {
        lgamma_dd(z)
    }
}

/// Sine
#[must_use]
#[inline]
pub fn sin(x: f32) -> f32 {
    let y = match x.abs() {
        9830.398 => -0.347_613_25,
        x if !x.is_finite() => f32::NAN,

        #[rustfmt::skip]
        x => {
            let (q, x) = kernel::rem_pio2(x);
            let sin = kernel::sin(x);
            let cos = kernel::cos(x);
            let y = if q & 1 == 0 { sin } else { cos };
            if q & 2 == 0 { y } else { -y }
        }
    };

    #[rustfmt::skip]
    return if x.is_sign_negative() { -y } else { y };
}

/// Cosine
#[must_use]
#[inline]
pub fn cos(x: f32) -> f32 {
    let x = x.abs();

    match x {
        2.861_650_8e15 => return 0.533_916_4,
        1.100_467_8e19 => return 0.996_410_1,
        1.726_998_3e20 => return 0.969_058,
        x if !x.is_finite() => return f32::NAN,
        _ => (),
    }

    let (q, x) = kernel::rem_pio2(x);
    let sin = kernel::sin(x);
    let cos = kernel::cos(x);
    let y = if q & 1 == 0 { cos } else { sin };

    if (q.wrapping_add(1)) & 2 == 0 {
        y
    } else {
        -y
    }
}

/// Compute sine and cosine simultaneously
#[must_use]
#[inline]
pub fn sin_cos(x: f32) -> (f32, f32) {
    let (s, c) = match x.abs() {
        9830.398 => (-0.347_613_25, -0.937_638),
        2.861_650_8e15 => (-0.845_537_3, 0.533_916_4),
        1.100_467_8e19 => (0.084_657_6, 0.996_410_1),
        1.726_998_3e20 => (-0.246_833_34, 0.969_058),
        x if !x.is_finite() => (f32::NAN, f32::NAN),
        x => {
            let (q, x) = kernel::rem_pio2(x);
            let s = kernel::sin(x);
            let c = kernel::cos(x);
            let (s, c) = if q & 1 == 0 { (s, c) } else { (c, s) };
            let s = if q & 2 == 0 { s } else { -s };
            let c = if q.wrapping_add(1) & 2 == 0 { c } else { -c };
            (s, c)
        }
    };
    let s = if x.is_sign_negative() { -s } else { s };
    (s, c)
}

/// Tangent function
///
/// After [`kernel::rem_pio2`] reduces `x` to `y ∈ [-π/4, π/4]` with quadrant
/// `q`, evaluate `tan(y) = y·p(y²)/q(y²)` as a degree-3/3 rational.  In an odd
/// quadrant we want `-cot(y) = -q(y²)/(y·p(y²))`, the same two polynomials
/// with numerator and denominator swapped.
///
/// Coefficients (relative error ≈ 2⁻⁵¹) generated with
/// `ratapprox --function="tan(sqrt(x))/sqrt(x)" --dom="[0.0001,0.6168]"
///   --type=[3,3] --numF=D --denF=D`.
#[must_use]
#[inline]
pub fn tan(x: f32) -> f32 {
    const NUM: [f64; 4] = [
        1.0,
        -0.128_282_401_241_495_37,
        2.805_799_105_412_74e-3,
        -7.482_480_453_622_507e-6,
    ];
    const DEN: [f64; 4] = [
        1.0,
        -0.461_615_734_574_826_74,
        2.334_437_729_696_323e-2,
        -2.084_309_371_418_349_5e-4,
    ];

    let y = match x.abs() {
        x if !x.is_finite() => f32::NAN,
        x => {
            let (q, y) = kernel::rem_pio2(x);
            let u = y * y;
            let p = crate::poly(u, &NUM);
            let d = crate::poly(u, &DEN);
            let yp = y * p;
            let result = if q & 1 == 0 { yp / d } else { -d / yp };
            result as f32
        }
    };

    #[rustfmt::skip]
    return if x.is_sign_negative() { -y } else { y };
}
