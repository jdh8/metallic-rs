mod common;

#[test]
fn test_erfc() {
    // Dense sweep of [−7, 28] covers the small kernel, the 2 − erfc reflection for
    // negative x, the whole positive tail down to the underflow threshold
    // (≈27.226), and the subnormal results near it; the bit sweep adds ±0,
    // subnormals, large magnitudes, NaN, ∞.
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 35.0 / 2_000_000.0, -7.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::erfc, core_math::erfc, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until erfc is
/// correctly rounded — issue #6).
#[test]
#[ignore = "faithful but not yet correctly rounded; tracked in issue #6"]
fn test_erfc_worst_cases() {
    common::test_worst_univariate("erfc", metallic::erfc, core_math::erfc);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_erfc_worst_faithful() {
    common::test_worst_faithful("erfc", metallic::erfc, core_math::erfc, 1);
}
