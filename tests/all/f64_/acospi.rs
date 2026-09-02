use crate::common;

#[test]
fn test_parser() {
    let count = common::parse_case_file("acospi.wc", common::parse_f64).count();
    assert!(count == 82_377, "parsed {count} cases");
}

#[test]
fn test_acospi() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        metallic::acospi,
        core_math::acospi,
        common::parse_case_file("acospi.wc", common::parse_f64).chain(bits),
    );
}

/// Dense sweep of the domain `[−1, 1]`, both branches.
#[test]
fn test_acospi_dense_band() {
    let lb = f64::MIN_POSITIVE.to_bits();
    let hb = 1.0_f64.to_bits();
    let band = (lb..=hb)
        .step_by(((hb - lb) / 4_000_000) as usize)
        .map(f64::from_bits);
    let cases = band.clone().chain(band.map(|x| -x));
    common::test_univariate_cases(metallic::acospi, core_math::acospi, cases);
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_acospi_worst_cases() {
    common::test_worst_univariate("acospi", metallic::acospi, core_math::acospi);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_acospi_worst_faithful() {
    common::test_worst_faithful("acospi", metallic::acospi, core_math::acospi, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_acospi_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).acos_pi().to_f64();
    let sampler = |i: u64| {
        let h = common::mix64(i);
        if i & 1 == 0 {
            common::uniform(h, -1.0, 1.0)
        } else {
            f64::from_bits(h)
        }
    };
    common::mpfr_sweep_univariate(metallic::acospi, cr, sampler, 2_000_000);
}
