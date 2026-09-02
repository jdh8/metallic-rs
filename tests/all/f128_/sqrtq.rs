use crate::common;
use crate::common128;

use common::Identity as _;

const SAMPLE_COUNT: u128 = 500_000;

fn dense_band() -> impl Iterator<Item = f128> {
    let lo = 1.0_f128.to_bits();
    let hi = 4.0_f128.to_bits();
    let step = (hi - lo) / SAMPLE_COUNT;
    (0..SAMPLE_COUNT).map(move |i| f128::from_bits(lo + i * step))
}

fn full_range() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT as u64).map(|i| f128::from_bits(common128::mix128(i)))
}

#[test]
fn test_parser() {
    assert!(common128::parse_f128("+snan").unwrap().is_nan());
    assert_eq!(common128::parse_f128("-0").unwrap().to_bits(), 1 << 127);
    assert_eq!(
        common128::parse_f128("+1").unwrap().to_bits(),
        1.0_f128.to_bits()
    );
    assert_eq!(common128::parse_f128("0x1p-16494").unwrap().to_bits(), 1);
    assert_eq!(
        common128::parse_f128("0x1.ffffffffffffffffffffffffffffp+16383")
            .unwrap()
            .to_bits(),
        f128::MAX.to_bits()
    );
    assert_eq!(
        common128::parse_f128("0x1.00000000000000000000000000008p0")
            .unwrap()
            .to_bits(),
        1.0_f128.to_bits()
    );
    assert_eq!(
        common128::parse_f128("0x1.00000000000000000000000000018p0")
            .unwrap()
            .to_bits(),
        1.0_f128.to_bits() + 2
    );
    assert_eq!(common128::parse_f128("0x1p-16495").unwrap().to_bits(), 0);
    assert_eq!(common128::parse_f128("0x3p-16495").unwrap().to_bits(), 2);
    assert!(
        common128::parse_f128("0x1.ffffffffffffffffffffffffffff8p+16383")
            .unwrap()
            .is_infinite()
    );
    assert_eq!(common128::ulp_error_f128(-0.0, 0.0), 1);
    assert_eq!(
        common::parse_case_file("sqrtq.wc", common128::parse_f128).count(),
        193_067
    );
}

#[test]
fn test_sqrtq() {
    common::test_univariate_cases(
        metallic::sqrtq,
        f128::sqrt,
        dense_band().chain(full_range()),
    );
}

#[test]
fn test_sqrtq_special() {
    assert!(metallic::sqrtq(-1.0).is_nan());
    assert!(metallic::sqrtq(-0.0).is(&-0.0));
    assert!(metallic::sqrtq(f128::INFINITY).is(&f128::INFINITY));
}

#[test]
fn test_sqrtq_worst_cases() {
    common128::test_worst_univariate_f128("sqrt", metallic::sqrtq, core_math::sqrtq);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_sqrtq_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::sqrtq,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.sqrt_round(Nearest)),
        |i| f128::from_bits(common128::mix128(i)),
        1_000_000,
    );
}
