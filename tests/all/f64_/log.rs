use crate::common;

/// Bit-pattern sweep plus a dense sweep of [0.5, 2] (the cancellation region near 1).
fn log_inputs() -> impl Iterator<Item = f64> {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_one = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1.5 / 2_000_000.0, 0.5));
    bits.chain(near_one)
}

#[test]
fn test_ln() {
    common::test_univariate_cases(metallic::log, core_math::log, log_inputs());
}

#[test]
fn test_log_worst_cases() {
    common::test_worst_univariate("log", metallic::log, core_math::log);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.  Representation-uniform over the
/// positive domain (sign bit cleared): +0, subnormals, normals, +∞, NaN.
#[cfg(feature = "mpfr")]
#[test]
fn test_log_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).ln().to_f64();
    common::mpfr_sweep_univariate(
        metallic::log,
        cr,
        |i| f64::from_bits(common::mix64(i) & 0x7FFF_FFFF_FFFF_FFFF),
        2_000_000,
    );
}
