mod common;

#[test]
fn test_parser() {
    let count = common::parse_case_file("exp2m1.wc", common::parse_f64).count();
    assert!(count == 240_833, "parsed {count} cases");
}

#[test]
fn test_exp2m1() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        metallic::exp2m1,
        core_math::exp2m1,
        common::parse_case_file("exp2m1.wc", common::parse_f64).chain(bits),
    );
}

/// Dense sweep across the small band, both signs, and the main band.
#[test]
fn test_exp2m1_dense_band() {
    let lb = f64::MIN_POSITIVE.to_bits();
    let hb = 60.0_f64.to_bits();
    let band = (lb..=hb)
        .step_by(((hb - lb) / 4_000_000) as usize)
        .map(f64::from_bits);
    let cases = band.clone().chain(band.map(|x| -x));
    common::test_univariate_cases(metallic::exp2m1, core_math::exp2m1, cases);
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_exp2m1_worst_cases() {
    common::test_worst_univariate("exp2m1", metallic::exp2m1, core_math::exp2m1);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_exp2m1_worst_faithful() {
    common::test_worst_faithful("exp2m1", metallic::exp2m1, core_math::exp2m1, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_exp2m1_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).exp2_m1().to_f64();
    let sampler = |i: u64| {
        let h = common::mix64(i);
        if i & 1 == 0 {
            common::uniform(h, -60.0, 1025.0)
        } else {
            f64::from_bits(h)
        }
    };
    common::mpfr_sweep_univariate(metallic::exp2m1, cr, sampler, 2_000_000);
}
