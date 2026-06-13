mod common;

#[test]
fn test_ln_1p() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_zero = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1.5 / 2_000_000.0, -0.5));
    common::test_univariate_cases(metallic::log1p, core_math::log1p, bits.chain(near_zero));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until log1p is
/// correctly rounded — issue #6).
#[test]
fn test_log1p_worst_cases() {
    common::test_worst_univariate("log1p", metallic::log1p, core_math::log1p);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_log1p_worst_faithful() {
    common::test_worst_faithful("log1p", metallic::log1p, core_math::log1p, 1);
}
