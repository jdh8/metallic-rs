use crate::common;

#[test]
fn test_erfc() {
    // Dense sweep of [−7, 28] covers the small kernel, the 2 − erfc reflection for
    // negative x, the whole positive tail down to the underflow threshold
    // (≈27.226), and the subnormal results near it; the bit sweep adds ±0,
    // subnormals, large magnitudes, NaN, ∞.
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 35.0 / 2_000_000.0, -7.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::erfc, core_math::erfc, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_erfc_worst_cases() {
    common::test_worst_univariate("erfc", metallic::erfc, core_math::erfc);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_erfc_worst_faithful() {
    common::test_worst_faithful("erfc", metallic::erfc, core_math::erfc, 1);
}

/// Independent confirmation of correct rounding against MPFR — the gold-standard
/// oracle CORE-MATH itself checks against (guards against a shared CORE-MATH
/// bug).  Run with `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_erfc_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).erfc().to_f64();
    // Value-uniform over [−6.25, 27.226]: the negative `2 − erfc` reflection, the
    // 1 − erf small region, the t-bridge, and the full positive tail down to the
    // underflow threshold (subnormal results included).
    common::mpfr_sweep_univariate(
        metallic::erfc,
        cr,
        |i| common::uniform(common::mix64(i), -6.25, 27.226_017_111_108_362),
        4_000_000,
    );
}
