mod common;

#[test]
fn test_erfc() {
    // Dense sweep of [−7, 28] covers the small kernel, the 2 − erfc reflection for
    // negative x, the whole positive tail down to the underflow threshold
    // (≈27.226), and the subnormal results near it; the bit sweep adds ±0,
    // subnormals, large magnitudes, NaN, ∞.
    let dense =
        (0..=2_000_000).map(|i| metallic::correct_mul_add(f64::from(i), 35.0 / 2_000_000.0, -7.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::erfc, core_math::erfc, dense.chain(bits));
}
