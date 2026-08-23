#![cfg(feature = "f128")]
#![cfg_attr(feature = "f128", feature(f128))]

mod common;
#[path = "common/f128.rs"]
mod common128;
#[path = "common/f128_exp.rs"]
mod common_exp;

use common::Identity as _;

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("exp10q.wc", common128::parse_f128).count(),
        PARSER_COUNT
    );
}

const PARSER_COUNT: usize = 20_422;

#[test]
fn test_exp10q() {
    common::test_univariate_cases(
        metallic::exp10q,
        core_math::exp10q,
        common_exp::dense(-4970.0, 4935.0).chain(common_exp::significands()),
    );
}

#[test]
fn test_exp10q_special() {
    assert!(metallic::exp10q(3.0).is(&1000.0));
    assert!(metallic::exp10q(f128::NEG_INFINITY).is(&0.0));
    assert!(metallic::exp10q(f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::exp10q(f128::NAN).is_nan());
}

#[test]
fn test_exp10q_worst_cases() {
    common128::test_worst_univariate_f128("exp10", metallic::exp10q, core_math::exp10q);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_exp10q_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::exp10q,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.exp10_round(Nearest)),
        common_exp::sampler(4935.0),
        200_000,
    );
}
