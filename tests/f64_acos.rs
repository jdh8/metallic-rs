mod common;
use metallic::f64 as metal;

#[test]
fn test_acos() {
    let dense = (0..=4_000_000).map(|i| f64::from(i).mul_add(2.0 / 4_000_000.0, -1.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::acos, core_math::acos, dense.chain(bits));
}
