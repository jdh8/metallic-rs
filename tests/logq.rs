#![cfg(feature = "f128")]
#![cfg_attr(feature = "f128", feature(f128))]

mod common;
#[path = "common/f128.rs"]
mod common128;

use common::Identity as _;

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("logq.wc", common128::parse_f128).count(),
        PARSER_COUNT
    );
}

const PARSER_COUNT: usize = 51_661;

const SAMPLE_COUNT: u64 = 200_000;

/// A random significand at an exponent uniform over every binade, subnormals
/// included: the whole domain of the logarithm.
fn positive(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let exponent = (bits >> 120) % 0x7fff;

    f128::from_bits(exponent << 112 | (bits & (1 << 112) - 1))
}

/// Bit-stepping sweep of the neighbourhood of 1, where the reduction cancels
/// and the accurate leg decides alone.
fn near_one() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT).flat_map(|i| {
        let up = 1.0_f128.to_bits() + u128::from(i);
        let down = 1.0_f128.to_bits() - u128::from(i);
        [f128::from_bits(up), f128::from_bits(down)]
    })
}

/// Uniform dense sweep of `(0, 2]`.
fn dense() -> impl Iterator<Item = f128> {
    (1..=SAMPLE_COUNT).map(|i| 2.0 * i as f128 / SAMPLE_COUNT as f128)
}

#[test]
fn test_logq() {
    common::test_univariate_cases(
        metallic::logq,
        core_math::logq,
        dense()
            .chain(near_one())
            .chain((0..SAMPLE_COUNT).map(positive)),
    );
}

#[test]
fn test_logq_special() {
    assert!(metallic::logq(1.0).is(&0.0));
    assert!(metallic::logq(0.0).is(&f128::NEG_INFINITY));
    assert!(metallic::logq(-0.0).is(&f128::NEG_INFINITY));
    assert!(metallic::logq(f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::logq(-1.0).is_nan());
    assert!(metallic::logq(f128::NEG_INFINITY).is_nan());
    assert!(metallic::logq(f128::NAN).is_nan());
}

#[test]
fn test_logq_worst_cases() {
    common128::test_worst_univariate_f128("log", metallic::logq, core_math::logq);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_logq_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::logq,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.ln_round(Nearest)),
        positive,
        SAMPLE_COUNT,
    );
}
