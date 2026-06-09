mod common;

/// Bit-pattern sweep plus a dense sweep of [0.5, 2] (the cancellation region near 1).
fn log_inputs() -> impl Iterator<Item = f64> {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_one = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1.5 / 2_000_000.0, 0.5));
    bits.chain(near_one)
}

#[test]
fn test_log10() {
    common::test_univariate_cases(metallic::log10, core_math::log10, log_inputs());
}
