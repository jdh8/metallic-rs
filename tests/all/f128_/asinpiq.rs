use crate::{common, common128, common128_arcpi as cases};
use common::Identity as _;

// Frozen precision-113 MPFR answers replay without the mpfr feature.
const CORPUS_LEN: usize = 239_107;
#[cfg(feature = "mpfr")]
const KIND: usize = 0;

#[test]
fn test_asinpiq_corpus() {
    let rows: Vec<[f128; 2]> =
        common::parse_case_file("asinpiq.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        rows.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );
    common::truncate_errors(rows.into_iter().filter_map(|[x, want]| {
        let got = metallic::asinpiq(x);
        (!got.is(&want)).then(|| println!("asinpiq({x:?}) = {got:?} != {want:?}"))
    }));
}

#[test]
fn test_asinpiq_special() {
    for x in [
        1.0_f128.next_up(),
        (-1.0_f128).next_down(),
        2.0,
        -2.0,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::NAN,
    ] {
        assert!(metallic::asinpiq(x).is_nan());
    }
    for x in [0.0_f128, -0.0] {
        assert!(metallic::asinpiq(x).is(&x));
    }
    assert_eq!(metallic::asinpiq(1.0), 0.5);
    assert_eq!(metallic::asinpiq(-1.0), -0.5);
    assert!(metallic::asinpiq(f128::from_bits(1)).is(&0.0));
    assert!(metallic::asinpiq(-f128::from_bits(1)).is(&-0.0));
    assert_eq!(metallic::asinpiq(0.5), 1.0 / 6.0);
    for x in [f128::from_bits(1), f128::MIN_POSITIVE, cases::power(-115)] {
        let y = metallic::asinpiq(x);
        assert!(y.is_finite());
        assert!(metallic::asinpiq(-x).is(&-y));
    }
}

/// Independent CORE-MATH f64 cross-check allowing one ulp for double rounding.
#[test]
fn test_asinpiq_vs_f64() {
    common::truncate_errors((0..200_000).filter_map(|i| {
        let b = common::mix64(i);
        let x = f64::from_bits(b & (1 << 63) | ((i % 0x3ff) << 52) | b & ((1 << 52) - 1));
        let got = metallic::asinpiq(x as f128) as f64;
        let want = core_math::asinpi(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("asinpiq({x:e}) = {got:e} != {want:e}"))
    }));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_asinpiq_vs_mpfr() {
    for sample in [
        cases::domain,
        cases::dense,
        cases::near_one,
        cases::subnormal,
        cases::seams,
        cases::small,
    ] {
        common128::mpfr_sweep_univariate_f128(
            metallic::asinpiq,
            |x| cases::oracle(KIND, x),
            sample,
            200_000,
        );
    }
    common::test_univariate_cases(
        metallic::asinpiq,
        |x| cases::oracle(KIND, x),
        cases::edges().into_iter(),
    );
}
