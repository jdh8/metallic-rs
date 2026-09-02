use crate::common;

#[test]
fn test_parser() {
    let count = common::parse_case_file("atanpi.wc", common::parse_f64).count();
    assert!(count == 114_383, "parsed {count} cases");
}

#[test]
fn test_atanpi() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        metallic::atanpi,
        core_math::atanpi,
        common::parse_case_file("atanpi.wc", common::parse_f64).chain(bits),
    );
}

/// Dense sweep across the tiny, small, table, and reflection regions.
#[test]
fn test_atanpi_dense_band() {
    let lb = f64::MIN_POSITIVE.to_bits();
    let hb = 1.0e17_f64.to_bits();
    let band = (lb..=hb)
        .step_by(((hb - lb) / 4_000_000) as usize)
        .map(f64::from_bits);
    let cases = band.clone().chain(band.map(|x| -x));
    common::test_univariate_cases(metallic::atanpi, core_math::atanpi, cases);
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_atanpi_worst_cases() {
    common::test_worst_univariate("atanpi", metallic::atanpi, core_math::atanpi);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_atanpi_worst_faithful() {
    common::test_worst_faithful("atanpi", metallic::atanpi, core_math::atanpi, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_atanpi_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).atan_pi().to_f64();
    common::mpfr_sweep_univariate(
        metallic::atanpi,
        cr,
        |i| f64::from_bits(common::mix64(i)),
        2_000_000,
    );
}
