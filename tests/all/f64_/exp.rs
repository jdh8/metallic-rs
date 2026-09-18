use crate::common;

#[test]
fn test_exp() {
    // Dense sweep of the finite range [−745, 710] plus bit-pattern stepping over
    // all of `f64` (covers ±0, subnormals, overflow/underflow, NaN, ∞).
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1455.0 / 2_000_000.0, -745.2));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::exp, core_math::exp, dense.chain(bits));
}

/// The subnormal-result band (issue #10's inline pre-scaled leg): a dense sweep
/// of `[-745.2, -708.3]`, every representation around the first normal input, and
/// the neighbourhood of every `2^-k` result boundary (`k = 1022..=1074`), where
/// the leg's scale `s` and the table index both change — bit-exact against the
/// oracle.
#[test]
fn test_exp_subnormal_band() {
    let width = -708.3_f64 - (-745.2_f64);
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), width / 2_000_000.0, -745.2));
    let threshold = f64::from_bits(0xc086_232b_dd7a_bcd2);
    let around = |c: f64| {
        let b = c.to_bits();
        (b - 512..=b + 512).map(f64::from_bits)
    };
    let boundaries = (1022..=1074).flat_map(|k| around(-f64::from(k) * core::f64::consts::LN_2));
    common::test_univariate_cases(
        metallic::exp,
        core_math::exp,
        dense.chain(around(threshold)).chain(boundaries),
    );
}

#[test]
fn test_exp_worst_cases() {
    common::test_worst_univariate("exp", metallic::exp, core_math::exp);
}

/// Independent confirmation of correct rounding against MPFR — the gold-standard
/// oracle CORE-MATH itself checks against (guards against a shared CORE-MATH
/// bug).  Run with `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_exp_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).exp().to_f64();
    common::mpfr_sweep_univariate(
        metallic::exp,
        cr,
        |i| common::uniform(common::mix64(i), -745.2, 709.8),
        2_000_000,
    );
}
