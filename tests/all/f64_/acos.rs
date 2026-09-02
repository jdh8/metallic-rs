use crate::common;

#[test]
fn test_acos() {
    let dense = (0..=4_000_000).map(|i| metallic::fma(f64::from(i), 2.0 / 4_000_000.0, -1.0));
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
        "0x1.390e6939cd1a6p-5",
        "-0x1.011c543f23a17p-2",
        "-0x1.4510ee8eb4e67p-1",
        "-0x1.771164bfd1f84p-3",
    ]
    .into_iter()
    .map(|s| common::parse_f64(s).unwrap());
    common::test_univariate_cases(
        metallic::acos,
        core_math::acos,
        dense.chain(bits).chain(hard),
    );
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_acos_worst_cases() {
    common::test_worst_univariate("acos", metallic::acos, core_math::acos);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_acos_worst_faithful() {
    common::test_worst_faithful("acos", metallic::acos, core_math::acos, 1);
}
