mod common;
use metallic::f64 as metal;

#[test]
fn test_acos() {
    let dense =
        (0..=4_000_000).map(|i| metallic::correct_mul_add(f64::from(i), 2.0 / 4_000_000.0, -1.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    // CORE-MATH's published hard-to-round acos arguments — the dense/bit sweeps
    // do not hit these exact patterns, and the worst (~2⁻¹¹¹ from a midpoint) is
    // beyond the double-double fallback's reach (see the triple-double escalation
    // in `src/f64/atan.rs`).
    let hard = [
        "0x1.53ea6c7255e88p-4",
        "0x1.fd737be914578p-11",
        "0x1.ffffffffffdc0p-1",
        "0x1.fffffffffff70p-1",
        "-0x1.011c543f23a17p-2",
        "-0x1.4510ee8eb4e67p-1",
        "-0x1.771164bfd1f84p-3",
    ]
    .into_iter()
    .map(|s| common::parse_f64(s).unwrap());
    common::test_univariate_cases(metal::acos, core_math::acos, dense.chain(bits).chain(hard));
}
