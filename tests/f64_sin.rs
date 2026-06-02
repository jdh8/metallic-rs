mod common;
use metallic::f64 as metal;

#[test]
fn test_sin() {
    // Dense over a few periods, a wide value sweep, and a representation-uniform
    // bit sweep (exercises the medium Cody–Waite and the Payne–Hanek paths).
    let dense = (0..=2_000_000).map(|i| f64::from(i).mul_add(20.0 / 2_000_000.0, -10.0));
    let wide = (0..=2_000_000).map(|i| f64::from(i).mul_add(2.0e8 / 2_000_000.0, -1.0e8));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::sin, core_math::sin, dense.chain(wide).chain(bits));
}
