mod common;

/// Bit-pattern sweep plus a dense sweep of [0.5, 2] (the cancellation region near 1).
fn log_inputs() -> impl Iterator<Item = f64> {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_one = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1.5 / 2_000_000.0, 0.5));
    bits.chain(near_one)
}

#[test]
fn test_log2() {
    common::test_univariate_cases(metallic::log2, core_math::log2, log_inputs());
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_log2_worst_cases() {
    common::test_worst_univariate("log2", metallic::log2, core_math::log2);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_log2_worst_faithful() {
    common::test_worst_faithful("log2", metallic::log2, core_math::log2, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.  Representation-uniform over the
/// positive domain (sign bit cleared): +0, subnormals, normals, +∞, NaN.
#[cfg(feature = "mpfr")]
#[test]
fn test_log2_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).log2().to_f64();
    common::mpfr_sweep_univariate(
        metallic::log2,
        cr,
        |i| f64::from_bits(common::mix64(i) & 0x7FFF_FFFF_FFFF_FFFF),
        2_000_000,
    );
}
