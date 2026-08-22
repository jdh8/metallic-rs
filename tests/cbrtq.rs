#![cfg(feature = "f128")]
#![cfg_attr(feature = "f128", feature(f128))]

mod common;
#[path = "common/f128.rs"]
mod common128;

use common::Identity as _;

const SAMPLE_COUNT: u128 = 500_000;

fn dense_band() -> impl Iterator<Item = f128> {
    let lo = 1.0_f128.to_bits();
    let hi = 8.0_f128.to_bits();
    let step = (hi - lo) / SAMPLE_COUNT;
    (0..SAMPLE_COUNT).map(move |i| f128::from_bits(lo + i * step))
}

fn full_range() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT as u64).map(|i| f128::from_bits(common128::mix128(i)))
}

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("cbrtq.wc", common128::parse_f128).count(),
        58_974
    );
}

#[test]
fn test_cbrtq() {
    common::test_univariate_cases(
        metallic::cbrtq,
        core_math::cbrtq,
        dense_band().chain(full_range()),
    );
}

#[test]
fn test_cbrtq_special() {
    assert!(metallic::cbrtq(-8.0).is(&-2.0));
    assert!(metallic::cbrtq(-0.0).is(&-0.0));
    assert!(metallic::cbrtq(f128::INFINITY).is(&f128::INFINITY));
}

#[test]
fn test_cbrtq_worst_cases() {
    common128::test_worst_univariate_f128("cbrt", metallic::cbrtq, core_math::cbrtq);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_cbrtq_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::cbrtq,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.cbrt_round(Nearest)),
        |i| f128::from_bits(common128::mix128(i)),
        1_000_000,
    );
}
