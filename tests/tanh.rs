mod common;

#[test]
fn test_tanh() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 50.0 / 2_000_000.0, -25.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::tanh, core_math::tanh, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus (RED until tanh is
/// correctly rounded — issue #6).
#[test]
fn test_tanh_worst_cases() {
    common::test_worst_univariate("tanh", metallic::tanh, core_math::tanh);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_tanh_worst_faithful() {
    common::test_worst_faithful("tanh", metallic::tanh, core_math::tanh, 1);
}

/// Independent confirmation of correct rounding against MPFR — the gold-standard
/// oracle CORE-MATH itself checks against.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_tanh_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).tanh().to_f64();
    // Exponent-uniform sampler over |x| ∈ [2⁻⁶⁰, ~2⁹) with a random sign: the
    // result-anchored small leg, the expm1(2x) cancellation band, and the
    // approach to ±1 are all exercised densely.
    let sampler = |i| {
        let h = common::mix64(i);
        let exp = 1023 - 60 + (h >> 52) % 70; // unbiased exponent in [-60, 9]
        let x = f64::from_bits((exp << 52) | (h & 0x000F_FFFF_FFFF_FFFF));
        if h & 1 == 1 { -x } else { x }
    };
    common::mpfr_sweep_univariate(metallic::tanh, cr, sampler, 5_000_000);
}
