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
#[inline]
pub fn exp(x: f64) -> f64 {
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

pub fn atanh_f32(x: f64) -> f64 {
    let y = x * x;
    let y = y * crate::f64::poly(
        y,
        &[
            0.333_333_881_908_101_6,
            0.199_887_707_877_885_12,
            0.149_358_638_794_499_98,
        ],
    );

    crate::f64::mul_add(y, x, x)
}
