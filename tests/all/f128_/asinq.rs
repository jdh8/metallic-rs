use crate::common;
use crate::common128;

use common::Identity as _;

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("asinq.wc", common128::parse_f128).count(),
        138_286
    );
}

const SAMPLE_COUNT: u64 = 200_000;

/// Mantissa-uniform arguments filling `(-1, 1)`: every binade, subnormals
/// included.
fn domain(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let exponent = (bits >> 112 & 0x7fff) % 0x3fff;
    f128::from_bits((bits & (1 << 127)) | exponent << 112 | (bits & (1 << 112) - 1))
}

/// A few ulps around the sector pullbacks `±sin(atan(k/64))` and
/// `±cos(atan(k/64))`, where the reduced tangent collapses and the wide
/// square root's slip matters most.
fn breakpoints() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT).map(|i| {
        let bits = common128::mix128(i);
        let k = (bits >> 113 & 63) as u32 + 1;
        let t = f128::from(k) / 64.0;
        let c = metallic::rsqrtq(t * t + 1.0);
        let x = if bits & (1 << 126) == 0 { t * c } else { c };
        let jitter = x.to_bits().wrapping_add(bits >> 119 & 15).wrapping_sub(7);
        f128::from_bits((bits & (1 << 127)) | jitter)
    })
}

/// A few ulps around the fast leg's own breakpoints `±j/128` and their
/// pullbacks `±√(1 − (j/128)²)`, where the sine difference collapses onto the
/// root's slip alone.
fn sine_breakpoints() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT).map(|i| {
        let bits = common128::mix128(i);
        let j = (bits >> 113 & 127) as u32 % 91 + 1;
        let t = f128::from(j) / 128.0;
        let x = if bits & (1 << 126) == 0 {
            t
        } else {
            metallic::sqrtq(1.0 - t * t)
        };
        let jitter = x.to_bits().wrapping_add(bits >> 119 & 15).wrapping_sub(7);
        f128::from_bits((bits & (1 << 127)) | jitter)
    })
}

/// The top binade approaching `±1`, where `1 − x²` cancels hardest.
fn near_one() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT).map(|i| {
        let bits = common128::mix128(i);
        let offset = (bits >> 88) % (1 << 24);
        f128::from_bits((bits & (1 << 127)) | (1.0_f128.to_bits() - 1 - offset))
    })
}

/// Uniform dense sweep of `[-1, 1]`.
fn dense() -> impl Iterator<Item = f128> {
    (0..=SAMPLE_COUNT).map(|i| 2.0 * i as f128 / SAMPLE_COUNT as f128 - 1.0)
}

#[test]
fn test_asinq() {
    common::test_univariate_cases(
        metallic::asinq,
        core_math::asinq,
        dense()
            .chain(breakpoints())
            .chain(sine_breakpoints())
            .chain(near_one())
            .chain((0..SAMPLE_COUNT).map(domain)),
    );
}

#[test]
fn test_asinq_special() {
    let half = core::f128::consts::FRAC_PI_2;
    assert!(metallic::asinq(0.0).is(&0.0));
    assert!(metallic::asinq(-0.0).is(&-0.0));
    assert!(metallic::asinq(1.0).is(&half));
    assert!(metallic::asinq(-1.0).is(&-half));
    assert!(metallic::asinq(0.5).is(&core::f128::consts::FRAC_PI_6));
    assert!(metallic::asinq(1.5).is_nan());
    assert!(metallic::asinq(f128::INFINITY).is_nan());
    assert!(metallic::asinq(f128::NEG_INFINITY).is_nan());
    assert!(metallic::asinq(f128::NAN).is_nan());
    // asin(x) rounds back to x below the cubic term's reach.
    let least = f128::from_bits(1);
    assert!(metallic::asinq(least).is(&least));
    assert!(metallic::asinq(f128::MIN_POSITIVE).is(&f128::MIN_POSITIVE));
}

#[test]
fn test_asinq_worst_cases() {
    common128::test_worst_univariate_f128("asin", metallic::asinq, core_math::asinq);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_asinq_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_univariate_f128(
        metallic::asinq,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.asin_round(Nearest)),
        domain,
        500_000,
    );
}
