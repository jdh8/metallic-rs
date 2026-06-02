mod common;
use metallic::f64 as metal;

#[test]
fn test_atanh() {
    // Uniform over (−1, 1), an extra sweep crowding the ±1 boundary (where the
    // result grows and rounding is hardest), and all bit patterns.
    let dense = (0..2_000_000).map(|i| f64::from(i).mul_add(2.0 / 2_000_000.0, -1.0));
    let near1 = (0..2_000_000u64).map(|i| {
        // Hashed mantissas in [0.5, 1), i.e. approaching 1 from below.
        let m = 0x3FE0_0000_0000_0000 | (i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 12);
        f64::from_bits(m)
    });
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        metal::atanh,
        core_math::atanh,
        dense.chain(near1).chain(bits),
    );
}
