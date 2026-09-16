use crate::{common, common128, common128_atanpi as cases};
use common::Identity as _;

// Frozen precision-113 MPFR answers, replayed without the mpfr feature.
const CORPUS_LEN: usize = 397_001;

#[test]
fn test_atan2piq_corpus() {
    let rows: Vec<_> =
        common::parse_case_file("atan2piq.wc", common128::parse_f128_triple).collect();
    assert_eq!(
        rows.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );
    common::truncate_errors(rows.into_iter().filter_map(|[y, x, want]| {
        let got = metallic::atan2piq(y, x);
        (!got.is(&want)).then(|| println!("atan2piq({y:?}, {x:?}) = {got:?} != {want:?}"))
    }));
}

#[test]
fn test_atan2piq_special() {
    for (y, x, want) in [
        (0.0_f128, 1.0_f128, 0.0_f128),
        (0.0, -1.0, 1.0),
        (0.0, 0.0, 0.0),
        (0.0, -0.0, 1.0),
        (1.0, 0.0, 0.5),
        (1.0, -0.0, 0.5),
        (1.0, 1.0, 0.25),
        (1.0, -1.0, 0.75),
        (1.0, f128::INFINITY, 0.0),
        (1.0, f128::NEG_INFINITY, 1.0),
        (f128::INFINITY, 1.0, 0.5),
        (f128::INFINITY, -1.0, 0.5),
        (f128::INFINITY, f128::INFINITY, 0.25),
        (f128::INFINITY, f128::NEG_INFINITY, 0.75),
        (f128::from_bits(1), 1.0, 0.0),
    ] {
        assert!(metallic::atan2piq(y, x).is(&want));
        assert!(metallic::atan2piq(-y, x).is(&-want));
    }
    for x in [0.0, -0.0, 1.0, -1.0, f128::INFINITY, f128::NAN] {
        assert!(metallic::atan2piq(x, f128::NAN).is_nan());
        assert!(metallic::atan2piq(f128::NAN, x).is_nan());
    }
    for x in [f128::from_bits(1), f128::MIN_POSITIVE, f128::MAX] {
        assert_eq!(metallic::atan2piq(x, x), 0.25);
        assert_eq!(metallic::atan2piq(x, -x), 0.75);
    }
}

#[test]
fn test_atan2piq_vs_f64() {
    common::truncate_errors((0..200_000).filter_map(|i| {
        let y = f64::from_bits(common::mix64(2 * i));
        let x = f64::from_bits(common::mix64(2 * i + 1));
        let got = metallic::atan2piq(y as f128, x as f128) as f64;
        let want = core_math::atan2pi(y, x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("atan2piq({y:e}, {x:e}) = {got:e} != {want:e}"))
    }));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_atan2piq_vs_mpfr() {
    for sample in [
        cases::pairs,
        cases::banded_pairs,
        cases::seam_pairs,
        cases::tiny_pairs,
    ] {
        common128::mpfr_sweep_bivariate_f128(metallic::atan2piq, cases::oracle2, sample, 200_000);
    }
    common128::test_bivariate_cases_f128(
        metallic::atan2piq,
        cases::oracle2,
        cases::edge_pairs().into_iter(),
    );
}
