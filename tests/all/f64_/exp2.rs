use crate::common;

#[test]
fn test_exp2() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 2099.0 / 2_000_000.0, -1075.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::exp2, core_math::exp2, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_exp2_worst_cases() {
    common::test_worst_univariate("exp2", metallic::exp2, core_math::exp2);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_exp2_worst_faithful() {
    common::test_worst_faithful("exp2", metallic::exp2, core_math::exp2, 1);
}

/// Independent confirmation of correct rounding against MPFR — the gold-standard
/// oracle CORE-MATH itself checks against (guards against a shared CORE-MATH
/// bug).  Run with `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_exp2_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).exp2().to_f64();
    common::mpfr_sweep_univariate(
        metallic::exp2,
        cr,
        |i| common::uniform(common::mix64(i), -1074.0, 1023.0),
        2_000_000,
    );
}
