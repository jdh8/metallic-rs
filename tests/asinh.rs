mod common;

#[test]
fn test_asinh() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 100.0 / 2_000_000.0, -50.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::asinh, core_math::asinh, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until asinh is
/// correctly rounded — issue #6).
#[test]
#[ignore = "faithful but not yet correctly rounded; tracked in issue #6"]
fn test_asinh_worst_cases() {
    common::test_worst_univariate("asinh", metallic::asinh, core_math::asinh);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_asinh_worst_faithful() {
    common::test_worst_faithful("asinh", metallic::asinh, core_math::asinh, 1);
}
