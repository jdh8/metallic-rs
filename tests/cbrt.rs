mod common;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("cbrt.wc", common::parse_f64).count() == 105_554);
}

#[test]
fn test_cbrt() {
    common::test_univariate_cases(
        metallic::cbrt,
        core_math::cbrt,
        common::parse_case_file("cbrt.wc", common::parse_f64)
            .chain((0..=u64::MAX).step_by((1 << 40) - 1337).map(f64::from_bits)),
    );
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.  Representation-uniform over all `f64`.
#[cfg(feature = "mpfr")]
#[test]
fn test_cbrt_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).cbrt().to_f64();
    common::mpfr_sweep_univariate(
        metallic::cbrt,
        cr,
        |i| f64::from_bits(common::mix64(i)),
        2_000_000,
    );
}
