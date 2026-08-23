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
        common::parse_case_file("expm1q.wc", common128::parse_f128).count(),
        PARSER_COUNT
    );
}

const PARSER_COUNT: usize = 27_378;

#[test]
fn test_expm1q() {
    common::test_univariate_cases(
        metallic::expm1q,
        core_math::expm1q,
        common_exp::dense(-120.0, 11400.0).chain(common_exp::significands()),
    );
}

#[test]
fn test_expm1q_special() {
    assert!(metallic::expm1q(0.0).is(&0.0));
    assert!(metallic::expm1q(-0.0).is(&-0.0));
    assert!(metallic::expm1q(f128::NEG_INFINITY).is(&-1.0));
    assert!(metallic::expm1q(f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::expm1q(f128::NAN).is_nan());
}

#[test]
fn test_expm1q_worst_cases() {
    common128::test_worst_univariate_f128("expm1", metallic::expm1q, core_math::expm1q);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_expm1q_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::expm1q,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.exp_m1_round(Nearest)),
        common_exp::exponents(),
        200_000,
    );
}
