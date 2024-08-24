/// Fast C `ldexp` assuming normal argument and result
#[inline]
pub fn fast_ldexp(x: f64, n: i64) -> f64 {
    const SHIFT: u32 = f64::MANTISSA_DIGITS - 1;

    #[allow(clippy::cast_possible_wrap)]
    let wrapped = x.to_bits() as i64;

    #[allow(clippy::cast_sign_loss)]
    return f64::from_bits((wrapped + (n << SHIFT)) as u64);
}

/// Polynomial approximation of restriction of `(exp(x) - 1) / x`
/// to `-0.5 * ln(2) ..= 0.5 * ln(2)`
///
/// In geometry, this function returns the slope of the secant line between the
/// points `(0, 1)` and `(x, exp(x))` on the graph of the exponential function.
#[inline]
pub fn exp_slope(x: f64) -> f64 {
    crate::poly(
        x,
        &[
            1.0,
            5.000_000_000_000_006e-1,
            1.666_666_666_666_660_7e-1,
            4.166_666_666_657_388_4e-2,
            8.333_333_333_377_178e-3,
            1.388_888_893_226_473e-3,
            1.984_126_974_692_237_6e-4,
            2.480_150_459_649_619_4e-5,
            2.755_738_188_469_386e-6,
            2.762_626_468_193_145e-7,
            2.506_206_487_727_576e-8,
        ],
    )
}

/// Polynomial approximation of inverse hyperbolic tangent restricted to
/// `-c..=c`, where
///
/// ```text
///     √2 - 1                  1 + c
/// c = ------  the solution to ----- = √2.
///     √2 + 1,                 1 - c
/// ```
#[inline]
pub fn atanh(x: f64) -> f64 {
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

    crate::mul_add(y, x, x)
}

/// Base 2 logarithm for a finite positive `f64`
#[inline]
pub fn log2(x: f64) -> f64 {
    use crate::f64::EXP_SHIFT;
    use core::f64::consts;

    #[allow(clippy::cast_possible_wrap)]
    let i = x.to_bits() as i64;

    #[allow(clippy::cast_possible_wrap)]
    let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let x = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);

    #[allow(clippy::cast_precision_loss)]
    crate::mul_add(
        2.0 * consts::LOG2_E,
        atanh((x - 1.0) / (x + 1.0)),
        exponent as f64,
    )
}

/// [`f64::exp2`] with precision of `f32`
///
/// This function is especially useful for computing `x`<sup>`y`</sup> when
/// combined with [`log2`].
#[inline]
pub fn exp2(x: f64) -> f64 {
    if x < (f64::MIN_EXP - 1).into() {
        return 0.0;
    }

    if x > f64::MAX_EXP.into() {
        return f64::INFINITY;
    }

    let n = x.round_ties_even();
    let x = crate::poly(
        x - n,
        &[
            1.0,
            6.931_471_880_289_533e-1,
            2.402_265_108_421_173_5e-1,
            5.550_357_105_498_874_4e-2,
            9.618_030_771_171_498e-3,
            1.339_086_685_300_951e-3,
            1.546_973_499_989_028_8e-4,
        ],
    );

    #[allow(clippy::cast_possible_truncation)]
    return fast_ldexp(x, n as i64);
}
