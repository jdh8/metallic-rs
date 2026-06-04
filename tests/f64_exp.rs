mod common;
use metallic::f64 as metal;

#[test]
fn test_exp() {
    // Dense sweep of the finite range [−745, 710] plus bit-pattern stepping over
    // all of `f64` (covers ±0, subnormals, overflow/underflow, NaN, ∞).
    let dense = (0..=2_000_000)
        .map(|i| metallic::correct_mul_add(f64::from(i), 1455.0 / 2_000_000.0, -745.2));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::exp, core_math::exp, dense.chain(bits));
}
