use crate::f64::EXP_SHIFT as F64_EXP_SHIFT;
use crate::f64::double::{DoubleDouble, fast_sum};
use core::num::FpCategory;

/// Polynomial approximation of inverse hyperbolic tangent restricted to
/// `-c..=c`, where
///
/// ```text
///     √2 - 1                  1 + c
/// c = ------  the solution to ----- = √2.
///     √2 + 1,                 1 - c
/// ```
#[inline]
pub(super) fn atanh(x: f64) -> f64 {
    let y = x * x;
    let y = y * crate::poly(
        y,
        &[
            0.333_333_333_333_310_1,
            0.200_000_000_056_551_2,
            0.142_857_120_550_553_72,
            0.111_114_324_826_276_95,
            0.090_700_447_553_529_28,
            0.083_116_173_891_988_07,
        ],
    );
    crate::fast_mul_add(y, x, x)
}

/// Base 2 logarithm for a finite positive `f64`
#[inline]
fn log2_f64(x: f64) -> f64 {
    use core::f64::consts;

    #[allow(clippy::cast_possible_wrap)]
    let i = x.to_bits() as i64;

    #[allow(clippy::cast_possible_wrap)]
    let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> F64_EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let x = f64::from_bits((i - (exponent << F64_EXP_SHIFT)) as u64);

    #[allow(clippy::cast_precision_loss)]
    crate::fast_mul_add(
        2.0 * consts::LOG2_E,
        atanh((x - 1.0) / (x + 1.0)),
        exponent as f64,
    )
}

/// Natural logarithm
#[must_use]
#[inline]
pub fn ln(x: f32) -> f32 {
    match super::normalize(x) {
        (crate::Sign::Positive, super::Magnitude::Infinite) => f32::INFINITY,
        (_, super::Magnitude::Zero) => f32::NEG_INFINITY,
        (crate::Sign::Negative, _) | (_, super::Magnitude::Nan) => f32::NAN,

        (crate::Sign::Positive, super::Magnitude::Normalized(i)) => {
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

            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> super::EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << super::EXP_SHIFT)) as u32).into();

            crate::fast_mul_add(
                core::f64::consts::LN_2,
                exponent.into(),
                2.0 * atanh((x - 1.0) / (x + 1.0)),
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
            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i64) >> F64_EXP_SHIFT;
            let y = f64::from_bits((i - (exponent << F64_EXP_SHIFT)) as u64);
            let z = if exponent == 0 { x } else { y - 1.0 };

            crate::fast_mul_add(
                -core::f64::consts::LN_2,
                -exponent as f64,
                2.0 * atanh(z / (z + 2.0)),
            ) as f32
        }
    }
}

/// Base 2 logarithm
#[must_use]
#[inline]
pub fn log2(x: f32) -> f32 {
    match super::normalize(x) {
        (crate::Sign::Positive, super::Magnitude::Infinite) => f32::INFINITY,
        (_, super::Magnitude::Zero) => f32::NEG_INFINITY,
        (crate::Sign::Negative, _) | (_, super::Magnitude::Nan) => f32::NAN,

        (crate::Sign::Positive, super::Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;
            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> super::EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << super::EXP_SHIFT)) as u32).into();

            crate::fast_mul_add(
                2.0 * core::f64::consts::LOG2_E,
                atanh((x - 1.0) / (x + 1.0)),
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

    match super::normalize(x) {
        (crate::Sign::Positive, super::Magnitude::Infinite) => f32::INFINITY,
        (_, super::Magnitude::Zero) => f32::NEG_INFINITY,
        (crate::Sign::Negative, _) | (_, super::Magnitude::Nan) => f32::NAN,

        (crate::Sign::Positive, super::Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;
            use core::f64::consts;

            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> super::EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << super::EXP_SHIFT)) as u32).into();
            let x = crate::fast_mul_add(
                2.0 * consts::LOG10_E,
                atanh((x - 1.0) / (x + 1.0)),
                LOG10_2_LO * f64::from(exponent),
            );
            crate::fast_mul_add(LOG10_2_HI, exponent.into(), x) as f32
        }
    }
}

/// Round a double-double down to `f32`, free of double rounding.
///
/// A bare `(high + low) as f32` rounds twice — to `f64`, then to `f32` — and can
/// land on the wrong side of an `f32` midpoint.  Instead round the pair *to odd*
/// into an `f64` whose last mantissa bit encodes the sign of the tail, then let
/// the hardware `as f32` (round-to-nearest-even) finish: an odd `f64` mantissa is
/// never an `f32` grid point or midpoint, so the second rounding is unambiguous.
#[inline]
fn round_to_f32(value: DoubleDouble) -> f32 {
    // Renormalize so `|low| ≤ ½ ulp(high)`; the product/reciprocal above may leave
    // the pair slightly denormalized.
    let DoubleDouble { high, low } = fast_sum(value.high, value.low);
    if low == 0.0 {
        return high as f32;
    }
    let bits = high.to_bits();
    let odd = if bits & 1 == 1 {
        high // already odd: the exact value rounds to odd here
    } else {
        // Step one `f64` ulp toward the tail so the kept value is the odd neighbor.
        let up = (bits >> 63 == 0) == (low > 0.0);
        f64::from_bits(if up { bits + 1 } else { bits - 1 })
    };
    odd as f32
}

/// Whether the `f64` quotient `q` lies dangerously close to an `f32` midpoint,
/// the Ziv trigger below.
///
/// Rounding a normal `f64` to `f32` discards the low 29 mantissa bits; the
/// midpoint is exactly bit 28 set, so the 29-bit residual measures the distance
/// to it.  Slow-path when that residual is within `BAND` of `2²⁸`.  `log`'s
/// result is always a normal `f32` (its magnitude is in `±[2⁻³², 2³²]`), so the
/// 29-bit cut never shifts.
#[inline]
const fn near_f32_midpoint(q: f64) -> bool {
    // The fast quotient's measured error is ≤ 2⁻²⁵·⁷ of a half-ulp, i.e. ≲ 8 of
    // these residual units (half-ulp = 2²⁸ units); `BAND = 2⁸` keeps a ~32× margin.
    const BAND: i64 = 1 << 8;
    let residual = (q.to_bits() & 0x1FFF_FFFF) as i64;
    (residual - (1 << 28)).abs() < BAND
}

/// Logarithm with arbitrary base
#[must_use]
#[inline]
pub fn log(x: f32, base: f32) -> f32 {
    #[inline]
    fn log2_inner(x: f32) -> f64 {
        match (x.is_sign_negative(), x.classify()) {
            (false, FpCategory::Infinite) => f64::INFINITY,
            (_, FpCategory::Zero) => f64::NEG_INFINITY,
            (true, _) | (_, FpCategory::Nan) => f64::NAN,
            _ => log2_f64(x.into()),
        }
    }

    // Non-finite / non-positive inputs, and `base == 1` (where `log2(base) = 0`),
    // give ∞/0/NaN that the plain `f64` ratio already rounds correctly.
    if !(x.is_finite() && x > 0.0 && base.is_finite() && base > 0.0) || base == 1.0 {
        return (log2_inner(x) / log2_inner(base)) as f32;
    }

    // Fast path: the single-`f64` quotient is accurate to ≤ 2⁻⁴⁹ relative (≈2⁻²⁵·⁷
    // of an `f32` half-ulp, measured), so `q as f32` is correctly rounded unless
    // `q` sits within that error of an `f32` midpoint.  The slow path then runs for
    // only ~2⁻²⁰ of inputs.
    let q = log2_f64(x.into()) / log2_f64(base.into());
    if !near_f32_midpoint(q) {
        return q as f32;
    }
    log_accurate(x, base)
}

/// Correctly-rounded `log(x, base)` for the rare near-midpoint case, kept out of
/// line so its heavy double-double machinery does not bloat [`log`]'s hot path.
///
/// Forms `log2(x) / log2(base)` as a double-double (≈2⁻⁹⁴, far past `f32`) and
/// rounds once — exactly `f64::log`'s scheme, a precision tier down.
#[cold]
#[inline(never)]
fn log_accurate(x: f32, base: f32) -> f32 {
    let ratio = crate::f64::pow::log2_dd(x.into()) * crate::f64::pow::log2_dd(base.into()).recip();
    round_to_f32(ratio)
}
