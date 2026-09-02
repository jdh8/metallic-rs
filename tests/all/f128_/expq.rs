use crate::common;
use crate::common_exp;
use crate::common128;

use common::Identity as _;

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("expq.wc", common128::parse_f128).count(),
        PARSER_COUNT
    );
}

const PARSER_COUNT: usize = 462_221;

#[test]
fn test_expq() {
    common::test_univariate_cases(
        metallic::expq,
        core_math::expq,
        common_exp::dense(-11400.0, 11400.0).chain(common_exp::significands()),
    );
}

#[test]
fn test_expq_special() {
    assert!(metallic::expq(0.0).is(&1.0));
    assert!(metallic::expq(f128::NEG_INFINITY).is(&0.0));
    assert!(metallic::expq(f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::expq(f128::NAN).is_nan());
}

#[test]
fn test_expq_worst_cases() {
    common128::test_worst_univariate_f128("exp", metallic::expq, core_math::expq);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_expq_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::expq,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.exp_round(Nearest)),
        common_exp::sampler(11400.0),
        200_000,
    );
}
