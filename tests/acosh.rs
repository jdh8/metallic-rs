mod common;

#[test]
fn test_acosh() {
    // Spread over [1, 1e6], dense just above 1 (the cancellation-prone region),
    // and a representation-uniform sweep over all bit patterns.
    let wide = (0..=2_000_000).map(|i| 1.0 + f64::from(i) * (1.0e6 / 2_000_000.0));
    let near1 = (0..=2_000_000).map(|i| 1.0 + f64::from(i) * (1.0e-9 / 2_000_000.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        metallic::acosh,
        core_math::acosh,
        wide.chain(near1).chain(bits),
    );
}
