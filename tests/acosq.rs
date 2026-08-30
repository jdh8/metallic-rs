#![cfg(feature = "f128")]
#![cfg_attr(feature = "f128", feature(f128))]

mod common;
#[path = "common/f128.rs"]
mod common128;

use common::Identity as _;

// The published `core-math` crate does not bind `acosq` yet, so until the next
// release the worst-case corpus and the sweeps run bit-exact against MPFR
// (`--features "f128 mpfr"`) — the same independent gold standard CORE-MATH
// checks itself with.  The parser count below runs unconditionally so the
// corpus cannot rot unnoticed.

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("acosq.wc", common128::parse_f128).count(),
        115_675
    );
}

const SAMPLE_COUNT: u64 = 200_000;

/// Mantissa-uniform arguments filling `(-1, 1)`: every binade, subnormals
/// included.
#[cfg(feature = "mpfr")]
fn domain(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let exponent = (bits >> 112 & 0x7fff) % 0x3fff;
    f128::from_bits((bits & (1 << 127)) | exponent << 112 | (bits & (1 << 112) - 1))
}

/// A few ulps around the sector pullbacks `±sin(atan(k/64))` and
/// `±cos(atan(k/64))`, where the reduced tangent collapses and the wide
/// square root's slip matters most.
#[cfg(feature = "mpfr")]
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

/// The top binade approaching `±1`: tiny positive results on one side, the
/// approach to π on the other.
#[cfg(feature = "mpfr")]
fn near_one() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT).map(|i| {
        let bits = common128::mix128(i);
        let offset = (bits >> 88) % (1 << 24);
        f128::from_bits((bits & (1 << 127)) | (1.0_f128.to_bits() - 1 - offset))
    })
}

/// Uniform dense sweep of `[-1, 1]`.
#[cfg(feature = "mpfr")]
fn dense() -> impl Iterator<Item = f128> {
    (0..=SAMPLE_COUNT).map(|i| 2.0 * i as f128 / SAMPLE_COUNT as f128 - 1.0)
}

#[cfg(feature = "mpfr")]
fn oracle(x: f128) -> f128 {
    use rug::float::Round::Nearest;

    metallic::f128_mpfr::cr_unop(x, |y| y.acos_round(Nearest))
}

#[cfg(feature = "mpfr")]
#[test]
fn test_acosq() {
    common::test_univariate_cases(
        metallic::acosq,
        oracle,
        dense()
            .chain(breakpoints())
            .chain(near_one())
            .chain((0..SAMPLE_COUNT).map(domain)),
    );
}

#[test]
fn test_acosq_special() {
    let half = core::f128::consts::FRAC_PI_2;
    assert!(metallic::acosq(1.0).is(&0.0));
    assert!(metallic::acosq(-1.0).is(&core::f128::consts::PI));
    assert!(metallic::acosq(0.0).is(&half));
    assert!(metallic::acosq(-0.0).is(&half));
    assert!(metallic::acosq(0.5).is(&core::f128::consts::FRAC_PI_3));
    assert!(metallic::acosq(1.5).is_nan());
    assert!(metallic::acosq(f128::INFINITY).is_nan());
    assert!(metallic::acosq(f128::NEG_INFINITY).is_nan());
    assert!(metallic::acosq(f128::NAN).is_nan());
    // A tiny argument leaves the rounding of π/2 untouched.
    assert!(metallic::acosq(f128::from_bits(1)).is(&half));
    assert!(metallic::acosq(-f128::MIN_POSITIVE).is(&half));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_acosq_worst_cases() {
    common128::test_worst_univariate_f128("acos", metallic::acosq, oracle);
}
