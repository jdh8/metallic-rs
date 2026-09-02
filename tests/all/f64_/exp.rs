use crate::common;

#[test]
fn test_exp() {
    // Dense sweep of the finite range [−745, 710] plus bit-pattern stepping over
    // all of `f64` (covers ±0, subnormals, overflow/underflow, NaN, ∞).
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1455.0 / 2_000_000.0, -745.2));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::exp, core_math::exp, dense.chain(bits));
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
