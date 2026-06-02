mod common;
use metallic::f64 as metal;

#[test]
fn test_atan() {
    let dense = (0..=2_000_000).map(|i| f64::from(i).mul_add(8.0 / 2_000_000.0, -4.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::atan, core_math::atan, dense.chain(bits));
}
