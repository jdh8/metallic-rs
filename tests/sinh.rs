mod common;

#[test]
fn test_sinh() {
    let dense = (0..=2_000_000)
        .map(|i| metallic::correct_mul_add(f64::from(i), 1422.0 / 2_000_000.0, -711.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::sinh, core_math::sinh, dense.chain(bits));
}
