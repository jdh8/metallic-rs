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
        common::parse_case_file("exp2q.wc", common128::parse_f128).count(),
        PARSER_COUNT
    );
}

const PARSER_COUNT: usize = 118_915;

#[test]
fn test_exp2q() {
    common::test_univariate_cases(
        metallic::exp2q,
        core_math::exp2q,
        common_exp::dense(-16500.0, 16400.0).chain(common_exp::significands()),
    );
}

#[test]
fn test_exp2q_special() {
    assert!(metallic::exp2q(10.0).is(&1024.0));
    assert!(metallic::exp2q(f128::NEG_INFINITY).is(&0.0));
    assert!(metallic::exp2q(f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::exp2q(f128::NAN).is_nan());
}

#[test]
fn test_exp2q_worst_cases() {
    common128::test_worst_univariate_f128("exp2", metallic::exp2q, core_math::exp2q);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_exp2q_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::exp2q,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.exp2_round(Nearest)),
        common_exp::sampler(16400.0),
        200_000,
    );
}
