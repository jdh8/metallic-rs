use crate::common;
use crate::common128;

use common::Identity as _;

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("atanq.wc", common128::parse_f128).count(),
        54_037
    );
}

const SAMPLE_COUNT: u64 = 200_000;

/// Representation-uniform bits: every binade, subnormals, and the specials.
fn wide(i: u64) -> f128 {
    f128::from_bits(common128::mix128(i))
}

/// Both signs in the kernel band around 1, where the reduction works hardest.
fn banded() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT).map(|i| {
        let bits = common128::mix128(i);
        let exponent = 0x3fff + (bits >> 112 & 0x7fff) % 41 - 20;
        f128::from_bits((bits & (1 << 127)) | exponent << 112 | (bits & (1 << 112) - 1))
    })
}

/// A few ulps around the breakpoint ratios `x = k/64` and their reciprocals,
/// where the reduced tangent collapses into nearly a pure table sum.
fn breakpoints() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT).map(|i| {
        let bits = common128::mix128(i);
        let k = (bits >> 113 & 63) as u32 + 1;
        let exact = f128::from(k) / 64.0;
        let x = if bits & (1 << 126) == 0 {
            exact
        } else {
            1.0 / exact
        };
        let jitter = x.to_bits().wrapping_add(bits >> 119 & 15).wrapping_sub(7);
        f128::from_bits((bits & (1 << 127)) | jitter)
    })
}

/// Uniform dense sweep of `[-4, 4]`.
fn dense() -> impl Iterator<Item = f128> {
    (0..=SAMPLE_COUNT).map(|i| 8.0 * i as f128 / SAMPLE_COUNT as f128 - 4.0)
}

#[test]
fn test_atanq() {
    common::test_univariate_cases(
        metallic::atanq,
        core_math::atanq,
        dense()
            .chain(banded())
            .chain(breakpoints())
            .chain((0..SAMPLE_COUNT).map(wide)),
    );
}

#[test]
fn test_atanq_special() {
    let half = core::f128::consts::FRAC_PI_2;
    assert!(metallic::atanq(0.0).is(&0.0));
    assert!(metallic::atanq(-0.0).is(&-0.0));
    assert!(metallic::atanq(1.0).is(&core::f128::consts::FRAC_PI_4));
    assert!(metallic::atanq(-1.0).is(&-core::f128::consts::FRAC_PI_4));
    assert!(metallic::atanq(f128::INFINITY).is(&half));
    assert!(metallic::atanq(f128::NEG_INFINITY).is(&-half));
    assert!(metallic::atanq(f128::NAN).is_nan());
    // atan(x) rounds back to x below the cubic term's reach.
    let least = f128::from_bits(1);
    assert!(metallic::atanq(least).is(&least));
    assert!(metallic::atanq(f128::MIN_POSITIVE).is(&f128::MIN_POSITIVE));
}

#[test]
fn test_atanq_worst_cases() {
    common128::test_worst_univariate_f128("atan", metallic::atanq, core_math::atanq);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_atanq_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::atanq,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.atan_round(Nearest)),
        wide,
        500_000,
    );
}
