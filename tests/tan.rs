mod common;

#[test]
fn test_tan() {
    // Dense over a few periods, a wide value sweep, and a representation-uniform
    // bit sweep (exercises the medium Cody–Waite and the Payne–Hanek paths).
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 20.0 / 2_000_000.0, -10.0));
    let wide = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 2.0e8 / 2_000_000.0, -1.0e8));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::tan, core_math::tan, dense.chain(wide).chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until tan is
/// correctly rounded — issue #6).
#[test]
fn test_tan_worst_cases() {
    common::test_worst_univariate("tan", metallic::tan, core_math::tan);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_tan_worst_faithful() {
    common::test_worst_faithful("tan", metallic::tan, core_math::tan, 1);
}
