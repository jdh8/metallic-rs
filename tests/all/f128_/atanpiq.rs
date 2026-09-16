use crate::{common, common128, common128_atanpi as cases};
use common::Identity as _;

// Frozen precision-113 MPFR answers, replayed without the mpfr feature.
const CORPUS_LEN: usize = 261_944;

#[test]
fn test_atanpiq_corpus() {
    let rows: Vec<_> = common::parse_case_file("atanpiq.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        rows.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );
    common::truncate_errors(rows.into_iter().filter_map(|[x, want]| {
        let got = metallic::atanpiq(x);
        (!got.is(&want)).then(|| println!("atanpiq({x:?}) = {got:?} != {want:?}"))
    }));
}

#[test]
fn test_atanpiq_special() {
    for (x, want) in [
        (0.0_f128, 0.0_f128),
        (1.0, 0.25),
        (f128::INFINITY, 0.5),
        (f128::MAX, 0.5),
        (f128::from_bits(1), 0.0),
        (f128::from_bits(2), f128::from_bits(1)),
    ] {
        assert!(metallic::atanpiq(x).is(&want));
        assert!(metallic::atanpiq(-x).is(&-want));
    }
    assert!(metallic::atanpiq(f128::NAN).is_nan());
}

#[test]
fn test_atanpiq_matches_atan2piq() {
    common::test_univariate_cases(
        metallic::atanpiq,
        |x| metallic::atan2piq(x, 1.0),
        (0..200_000).flat_map(|i| {
            [
                cases::domain(i),
                cases::banded(i),
                cases::seams(i),
                cases::subnormal(i),
            ]
        }),
    );
}

/// The f64 oracle is independent; one ulp allows the final double rounding.
#[test]
fn test_atanpiq_vs_f64() {
    common::truncate_errors((0..200_000).filter_map(|i| {
        let x = f64::from_bits(common::mix64(i));
        let got = metallic::atanpiq(x as f128) as f64;
        let want = core_math::atanpi(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("atanpiq({x:e}) = {got:e} != {want:e}"))
    }));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_atanpiq_vs_mpfr() {
    for sample in [cases::domain, cases::banded, cases::seams, cases::subnormal] {
        common128::mpfr_sweep_univariate_f128(metallic::atanpiq, cases::oracle, sample, 200_000);
    }
    common::test_univariate_cases(metallic::atanpiq, cases::oracle, cases::edges().into_iter());
}
