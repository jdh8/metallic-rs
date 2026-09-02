use crate::common;

#[test]
fn test_asinh() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 100.0 / 2_000_000.0, -50.0));
    // Bit-step through the sqrt-free asymptotic band [64, 2²⁷) (its Ziv gate is
    // tightest at the bottom, x = 64) — equal bit steps give each binade an equal
    // share, so the critical [64, 128) bottom binade is sampled densely.
    let asymptotic = (64.0_f64.to_bits()..134_217_728.0_f64.to_bits())
        .step_by((1 << 35) + 1)
        .map(f64::from_bits);
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        metallic::asinh,
        core_math::asinh,
        dense.chain(asymptotic).chain(bits),
    );
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_asinh_worst_cases() {
    common::test_worst_univariate("asinh", metallic::asinh, core_math::asinh);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_asinh_worst_faithful() {
    common::test_worst_faithful("asinh", metallic::asinh, core_math::asinh, 1);
}
