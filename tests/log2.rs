mod common;

/// Bit-pattern sweep plus a dense sweep of [0.5, 2] (the cancellation region near 1).
fn log_inputs() -> impl Iterator<Item = f64> {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_one = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1.5 / 2_000_000.0, 0.5));
    bits.chain(near_one)
}

#[test]
fn test_log2() {
    common::test_univariate_cases(metallic::log2, core_math::log2, log_inputs());
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until log2 is
/// correctly rounded — issue #6).
#[test]
#[ignore = "faithful but not yet correctly rounded; tracked in issue #6"]
fn test_log2_worst_cases() {
    common::test_worst_univariate("log2", metallic::log2, core_math::log2);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_log2_worst_faithful() {
    common::test_worst_faithful("log2", metallic::log2, core_math::log2, 1);
}
