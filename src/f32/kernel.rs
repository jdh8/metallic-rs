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
/// to [-0.5 ln 2, 0.5 ln 2]
///
/// In geometry, this function returns the slope of the secant line between the
/// points `(0, 1)` and `(x, exp(x))` on the graph of the exponential function.
#[inline]
pub fn exp_slope(x: f64) -> f64 {
    crate::f64::poly(
        x,
        &[
            1.000_000_010_775_500_7,
            5.000_000_080_819_904e-1,
            1.666_650_523_422_326_7e-1,
            4.166_624_066_361_261e-2,
            8.369_150_671_031_009e-3,
            1.394_858_354_331_218_4e-3,
        ],
    )
}

/// Polynomial approximation of inverse hyperbolic tangent restricted to [-c,
/// c], where
///
/// ```text
///     √2 - 1                  1 + c
/// c = ------  the solution to ----- = √2.
///     √2 + 1,                 1 - c
/// ```
#[inline]
pub fn atanh(x: f64) -> f64 {
    let y = x * x;
    let y = y * crate::f64::poly(
        y,
        &[
            0.333_333_426_330_174_6,
            0.199_943_594_363_205_02,
            0.147_910_234_311_395_7,
        ],
    );

    crate::f64::mul_add(y, x, x)
}

/// Base 2 logarithm for a finite positive `f64`
#[inline]
pub fn log2(x: f64) -> f64 {
    use crate::f64::EXP_SHIFT;
    use core::f64::consts;

    fn atanh(x: f64) -> f64 {
        let y = x * x;
        let y = y * crate::f64::poly(
            y,
            &[
                0.333_333_328_227_282_3,
                0.200_001_675_954_362_63,
                0.142_686_542_711_886_85,
                0.117_910_756_496_814_14,
            ],
        );

        crate::f64::mul_add(y, x, x)
    }

    #[allow(clippy::cast_possible_wrap)]
    let i = x.to_bits() as i64;

    #[allow(clippy::cast_possible_wrap)]
    let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let x = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);

    #[allow(clippy::cast_precision_loss)]
    crate::f64::mul_add(
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
    let x = crate::f64::poly(
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
