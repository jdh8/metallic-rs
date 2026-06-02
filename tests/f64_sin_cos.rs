mod common;
use metallic::f64 as metal;

#[test]
fn test_sin_cos() {
    // No f64 core-math `sincos`; check each component against `sin`/`cos`.
    let dense = (0..=2_000_000).map(|i| f64::from(i).mul_add(20.0 / 2_000_000.0, -10.0));
    let bits = (0..=u64::MAX).step_by((1 << 39) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        |x| metal::sin_cos(x).0,
        core_math::sin,
        dense.clone().chain(bits.clone()),
    );
    common::test_univariate_cases(|x| metal::sin_cos(x).1, core_math::cos, dense.chain(bits));
}
