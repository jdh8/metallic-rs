/// Fast C `ldexp` assuming normal argument and result
#[inline]
pub fn fast_ldexp(x: f64, n: i64) -> f64 {
    const SHIFT: u32 = f64::MANTISSA_DIGITS - 1;

    #[allow(clippy::cast_possible_wrap)]
    let wrapped = x.to_bits() as i64;

    #[allow(clippy::cast_sign_loss)]
    return f64::from_bits((wrapped + (n << SHIFT)) as u64);
}

/// Restriction of `(exp(x) - 1) / x` to [-0.5 ln 2, 0.5 ln 2]
#[inline]
pub fn exp(x: f64) -> f64 {
    #[allow(clippy::excessive_precision)]
    const P: [f64; 6] = [
        1.000_000_010_775_500_705,
        5.000_000_080_819_903_627e-1,
        1.666_650_523_422_326_531e-1,
        4.166_624_066_361_261_157e-2,
        8.369_150_671_031_008_566e-3,
        1.394_858_354_331_218_335e-3,
    ];

    P[5].mul_add(x, P[4])
        .mul_add(x, P[3])
        .mul_add(x, P[2])
        .mul_add(x, P[1])
        .mul_add(x, P[0])
}
