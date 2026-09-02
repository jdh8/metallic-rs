use crate::common;
use crate::common128;

use common::Identity as _;

const SAMPLE_COUNT: u64 = 200_000;
const FRACTION: u128 = (1 << 112) - 1;
const SIGN: u128 = 1 << 127;

/// Both magnitudes in [1, 2) with random signs: every sector, both sides of
/// the swap, and all four quadrants.
fn similar() -> impl Iterator<Item = [f128; 2]> {
    let leg = |i| f128::from_bits(1.0_f128.to_bits() | (common128::mix128(i) & (SIGN | FRACTION)));
    (0..SAMPLE_COUNT).map(move |i| [leg(2 * i), leg(2 * i + 1)])
}

/// Representation-uniform pairs: huge exponent gaps, subnormals, and the
/// ∞/NaN/zero specials.
fn wide() -> impl Iterator<Item = [f128; 2]> {
    let leg = |i| f128::from_bits(common128::mix128(i));
    (0..SAMPLE_COUNT).map(move |i| [leg(2 * i), leg(2 * i + 1)])
}

/// Exponents within ±20 of each other: the band where the reduction works
/// hardest, matching the benchmark draws.
fn banded() -> impl Iterator<Item = [f128; 2]> {
    (0..SAMPLE_COUNT).map(|i| {
        let a = common128::mix128(2 * i);
        let b = common128::mix128(2 * i + 1);
        let ea = 0x3fff + (a >> 112 & 0x7fff) % 41 - 20;
        let eb = 0x3fff + (b >> 112 & 0x7fff) % 41 - 20;
        [
            f128::from_bits((a & SIGN) | ea << 112 | (a & FRACTION)),
            f128::from_bits((b & SIGN) | eb << 112 | (b & FRACTION)),
        ]
    })
}

/// A few ulps around the exact breakpoint ratios `y/x = k/64`, where the
/// reduced tangent collapses and the result is nearly a pure table sum.
fn breakpoints() -> impl Iterator<Item = [f128; 2]> {
    (0..SAMPLE_COUNT).map(|i| {
        let bits = common128::mix128(i);
        let x = f128::from_bits(1.0_f128.to_bits() | (bits & FRACTION));
        let k = (bits >> 113 & 63) as u32 + 1;
        let near = (x / 64.0 * f128::from(k)).to_bits();
        let jitter = near.wrapping_add(bits >> 119 & 15).wrapping_sub(7);
        [
            f128::from_bits((bits & SIGN) | jitter),
            f128::from_bits((bits >> 1 & SIGN) | x.to_bits()),
        ]
    })
}

/// Tiny over huge and huge over tiny: subnormal results, underflow to zero,
/// and the approach to ±π/2 and ±π.
fn axes() -> impl Iterator<Item = [f128; 2]> {
    (0..SAMPLE_COUNT).map(|i| {
        let a = common128::mix128(2 * i);
        let b = common128::mix128(2 * i + 1);
        let tiny = f128::from_bits((a & SIGN) | (a & (FRACTION >> 4)));
        let big =
            f128::from_bits((b & SIGN) | (0x7ffe - (b >> 112 & 0xfff)) << 112 | (b & FRACTION));
        if i % 2 == 0 { [tiny, big] } else { [big, tiny] }
    })
}

/// The C99 special-value matrix and the extremes of the grid.
fn edges() -> impl Iterator<Item = [f128; 2]> {
    let specials = [
        0.0_f128,
        -0.0,
        1.0,
        -1.0,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::NAN,
        f128::MAX,
        f128::MIN_POSITIVE,
        f128::from_bits(1),
        f128::from_bits(SIGN | 1),
        f128::from_bits(FRACTION),
    ];
    specials
        .into_iter()
        .flat_map(move |y| specials.into_iter().map(move |x| [y, x]))
}

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("atan2q.wc", common128::parse_f128_pair).count(),
        269_282
    );
}

#[test]
fn test_atan2q() {
    common128::test_bivariate_cases_f128(
        metallic::atan2q,
        core_math::atan2q,
        similar()
            .chain(wide())
            .chain(banded())
            .chain(breakpoints())
            .chain(axes())
            .chain(edges()),
    );
}

#[test]
fn test_atan2q_special() {
    let pi = core::f128::consts::PI;
    assert!(metallic::atan2q(0.0, 1.0).is(&0.0));
    assert!(metallic::atan2q(-0.0, 1.0).is(&-0.0));
    assert!(metallic::atan2q(0.0, -1.0).is(&pi));
    assert!(metallic::atan2q(-0.0, -1.0).is(&-pi));
    assert!(metallic::atan2q(0.0, 0.0).is(&0.0));
    assert!(metallic::atan2q(0.0, -0.0).is(&pi));
    assert!(metallic::atan2q(1.0, 0.0).is(&core::f128::consts::FRAC_PI_2));
    assert!(metallic::atan2q(-1.0, -0.0).is(&-core::f128::consts::FRAC_PI_2));
    assert!(metallic::atan2q(1.0, 1.0).is(&core::f128::consts::FRAC_PI_4));
    // 3π/4: scaling the rounded π by 3/4 would double-round, so spell the bits.
    assert!(
        metallic::atan2q(f128::INFINITY, f128::NEG_INFINITY)
            .is(&f128::from_bits(0x4000_2d97_c7f3_321d_234f_2729_93d1_414a))
    );
    assert!(metallic::atan2q(f128::NAN, 1.0).is_nan());
    assert!(metallic::atan2q(1.0, f128::NAN).is_nan());
}

#[test]
fn test_atan2q_worst_cases() {
    common128::test_worst_bivariate_f128("atan2", metallic::atan2q, core_math::atan2q);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_atan2q_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_bivariate_f128(
        metallic::atan2q,
        |y, x| metallic::f128_mpfr::cr_binop(y, x, |a, b| a.atan2_round(b, Nearest)),
        |i| {
            [
                f128::from_bits(common128::mix128(2 * i)),
                f128::from_bits(common128::mix128(2 * i + 1)),
            ]
        },
        500_000,
    );
}
