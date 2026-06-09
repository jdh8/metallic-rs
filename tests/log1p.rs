mod common;

#[test]
fn test_ln_1p() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_zero =
        (0..=2_000_000).map(|i| metallic::correct_mul_add(f64::from(i), 1.5 / 2_000_000.0, -0.5));
    common::test_univariate_cases(metallic::log1p, core_math::log1p, bits.chain(near_zero));
}
