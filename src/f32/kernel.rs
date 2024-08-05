/// Fast C `ldexp` assuming normal argument and result
#[allow(clippy::cast_sign_loss)]
pub fn fast_ldexp(x: f64, n: i32) -> f64 {
    f64::from_bits((i64::from(n) << 52) as u64 + x.to_bits())
}

/// Restriction of `x.exp_m1() / x` to [-0.5 ln 2, 0.5 ln 2]
pub fn exp(x: f64) -> f64 {
    #[allow(clippy::excessive_precision, clippy::unreadable_literal)]
    const C: [f64; 6] = [
        1.000000010775500705,
        5.000000080819903627e-1,
        1.666650523422326531e-1,
        4.166624066361261157e-2,
        8.369150671031008566e-3,
        1.394858354331218335e-3
    ];

    C[5].mul_add(x, C[4])
        .mul_add(x, C[3])
        .mul_add(x, C[2])
        .mul_add(x, C[1])
        .mul_add(x, C[0])
}