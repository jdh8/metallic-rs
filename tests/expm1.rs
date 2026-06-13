mod common;

#[test]
fn test_exp_m1() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1420.0 / 2_000_000.0, -710.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::expm1, core_math::expm1, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until expm1 is
/// correctly rounded — issue #6).
#[test]
fn test_expm1_worst_cases() {
    common::test_worst_univariate("expm1", metallic::expm1, core_math::expm1);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_expm1_worst_faithful() {
    common::test_worst_faithful("expm1", metallic::expm1, core_math::expm1, 1);
}
