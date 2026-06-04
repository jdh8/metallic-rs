mod common;
use metallic::f64 as metal;

#[test]
fn test_exp_m1() {
    let dense = (0..=2_000_000)
        .map(|i| metallic::correct_mul_add(f64::from(i), 1420.0 / 2_000_000.0, -710.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::exp_m1, core_math::expm1, dense.chain(bits));
}
