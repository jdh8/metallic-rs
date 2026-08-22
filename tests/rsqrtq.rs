#![cfg(feature = "f128")]
#![cfg_attr(feature = "f128", feature(f128))]

mod common;
#[path = "common/f128.rs"]
mod common128;

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
    assert_eq!(
        common::parse_case_file("rsqrtq.wc", common128::parse_f128).count(),
        92_460
    );
}

#[test]
fn test_rsqrtq() {
    common::test_univariate_cases(
        metallic::rsqrtq,
        core_math::rsqrtq,
        dense_band().chain(full_range()),
    );
}

#[test]
fn test_rsqrtq_special() {
    assert!(metallic::rsqrtq(-1.0).is_nan());
    assert!(metallic::rsqrtq(0.0).is(&f128::INFINITY));
    assert!(metallic::rsqrtq(-0.0).is(&f128::NEG_INFINITY));
    assert!(metallic::rsqrtq(f128::INFINITY).is(&0.0));
}

#[test]
fn test_rsqrtq_worst_cases() {
    common128::test_worst_univariate_f128("rsqrt", metallic::rsqrtq, core_math::rsqrtq);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_rsqrtq_vs_mpfr() {
    use rug::float::Round::Nearest;

    let cr = |x: f128| {
        if x.to_bits() == 1 << 127 {
            f128::NEG_INFINITY
        } else {
            metallic::f128_mpfr::cr_unop(x, |y| y.recip_sqrt_round(Nearest))
        }
    };
    common128::mpfr_sweep_univariate_f128(
        metallic::rsqrtq,
        cr,
        |i| f128::from_bits(common128::mix128(i)),
        1_000_000,
    );
}
