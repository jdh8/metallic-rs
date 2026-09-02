use crate::common;

#[test]
fn test_exp10() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 632.0 / 2_000_000.0, -323.7));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::exp10, core_math::exp10, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_exp10_worst_cases() {
    common::test_worst_univariate("exp10", metallic::exp10, core_math::exp10);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_exp10_worst_faithful() {
    common::test_worst_faithful("exp10", metallic::exp10, core_math::exp10, 1);
}

/// Independent confirmation of correct rounding against MPFR — the gold-standard
/// oracle CORE-MATH itself checks against (guards against a shared CORE-MATH
/// bug).  Run with `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_exp10_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).exp10().to_f64();
    common::mpfr_sweep_univariate(
        metallic::exp10,
        cr,
        |i| common::uniform(common::mix64(i), -323.7, 308.3),
        2_000_000,
    );
}
