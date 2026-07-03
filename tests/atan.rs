mod common;

#[test]
fn test_atan() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 8.0 / 2_000_000.0, -4.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::atan, core_math::atan, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_atan_worst_cases() {
    common::test_worst_univariate("atan", metallic::atan, core_math::atan);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_atan_worst_faithful() {
    common::test_worst_faithful("atan", metallic::atan, core_math::atan, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_atan_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).atan().to_f64();
    common::mpfr_sweep_univariate(
        metallic::atan,
        cr,
        // A value-uniform spread over a wide magnitude band (both signs), so the
        // sweep exercises the direct cell kernel, the |x| > 1 reflection, and the
        // tiny-x fast return alike.
        |i| {
            let u = common::uniform(common::mix64(i), -1.0, 1.0);
            u * (2.0f64).powi((common::mix64(i ^ 0x5bd1_e995) % 200) as i32 - 100)
        },
        2_000_000,
    );
}
