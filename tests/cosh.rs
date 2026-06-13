mod common;

#[test]
fn test_cosh() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1422.0 / 2_000_000.0, -711.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::cosh, core_math::cosh, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until cosh is
/// correctly rounded — issue #6).
#[test]
#[ignore = "faithful but not yet correctly rounded; tracked in issue #6"]
fn test_cosh_worst_cases() {
    common::test_worst_univariate("cosh", metallic::cosh, core_math::cosh);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_cosh_worst_faithful() {
    common::test_worst_faithful("cosh", metallic::cosh, core_math::cosh, 1);
}
