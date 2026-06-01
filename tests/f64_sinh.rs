mod common;
use metallic::f64 as metal;

#[test]
fn test_sinh() {
    let dense = (0..=2_000_000).map(|i| f64::from(i).mul_add(1422.0 / 2_000_000.0, -711.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::sinh, core_math::sinh, dense.chain(bits));
}
