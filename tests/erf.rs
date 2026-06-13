mod common;

#[test]
fn test_erf() {
    // Dense sweep of [−7, 7] (small kernel, the erfc bridge, and the ±1 threshold
    // at |x| ≈ 6.25) plus a representation-uniform bit sweep over all of `f64`
    // (covers ±0, subnormals, large magnitudes, NaN, ∞).
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 14.0 / 2_000_000.0, -7.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::erf, core_math::erf, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until erf is
/// correctly rounded — issue #6).
#[test]
fn test_erf_worst_cases() {
    common::test_worst_univariate("erf", metallic::erf, core_math::erf);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_erf_worst_faithful() {
    common::test_worst_faithful("erf", metallic::erf, core_math::erf, 1);
}
