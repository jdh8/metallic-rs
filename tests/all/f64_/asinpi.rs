use crate::common;

#[test]
fn test_parser() {
    let count = common::parse_case_file("asinpi.wc", common::parse_f64).count();
    assert!(count == 58_408, "parsed {count} cases");
}

#[test]
fn test_asinpi() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        metallic::asinpi,
        core_math::asinpi,
        common::parse_case_file("asinpi.wc", common::parse_f64).chain(bits),
    );
}

/// Dense sweep of the domain `[−1, 1]`, both branches and the tiny band.
#[test]
fn test_asinpi_dense_band() {
    let lb = f64::MIN_POSITIVE.to_bits();
    let hb = 1.0_f64.to_bits();
    let band = (lb..=hb)
        .step_by(((hb - lb) / 4_000_000) as usize)
        .map(f64::from_bits);
    let cases = band.clone().chain(band.map(|x| -x));
    common::test_univariate_cases(metallic::asinpi, core_math::asinpi, cases);
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_asinpi_worst_cases() {
    common::test_worst_univariate("asinpi", metallic::asinpi, core_math::asinpi);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_asinpi_worst_faithful() {
    common::test_worst_faithful("asinpi", metallic::asinpi, core_math::asinpi, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.  Value-uniform over the domain.
#[cfg(feature = "mpfr")]
#[test]
fn test_asinpi_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).asin_pi().to_f64();
    let sampler = |i: u64| {
        let h = common::mix64(i);
        if i & 1 == 0 {
            common::uniform(h, -1.0, 1.0)
        } else {
            f64::from_bits(h) // full domain, incl. |x| > 1, ∞, NaN
        }
    };
    common::mpfr_sweep_univariate(metallic::asinpi, cr, sampler, 2_000_000);
}
