use crate::common;

#[test]
fn test_exp2() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 2099.0 / 2_000_000.0, -1075.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::exp2, core_math::exp2, dense.chain(bits));
}

/// The subnormal-result band (issue #10's inline pre-scaled leg): a dense sweep
/// of `[-1075.0, -1021.9]`, every representation around the first normal input, and
/// the neighbourhood of every `2^-k` result boundary (`k = 1022..=1074`), where
/// the leg's scale `s` and the table index both change — bit-exact against the
/// oracle.
#[test]
fn test_exp2_subnormal_band() {
    let width = -1021.9_f64 - (-1075.0_f64);
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), width / 2_000_000.0, -1075.0));
    let threshold = -1022.0;
    let around = |c: f64| {
        let b = c.to_bits();
        (b - 512..=b + 512).map(f64::from_bits)
    };
    let boundaries = (1022..=1074).flat_map(|k| around(-f64::from(k) * 1.0));
    common::test_univariate_cases(
        metallic::exp2,
        core_math::exp2,
        dense.chain(around(threshold)).chain(boundaries),
    );
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
