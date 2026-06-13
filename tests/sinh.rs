mod common;

#[test]
fn test_sinh() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1422.0 / 2_000_000.0, -711.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::sinh, core_math::sinh, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until sinh is
/// correctly rounded — issue #6).
#[test]
fn test_sinh_worst_cases() {
    common::test_worst_univariate("sinh", metallic::sinh, core_math::sinh);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_sinh_worst_faithful() {
    common::test_worst_faithful("sinh", metallic::sinh, core_math::sinh, 1);
}
