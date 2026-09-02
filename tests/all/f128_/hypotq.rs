use crate::common;
use crate::common128;

use common::Identity as _;

const SAMPLE_COUNT: u64 = 250_000;
const FRACTION: u128 = (1 << 112) - 1;

/// Both legs in `[1, 2)`: `a² + b²` covers `[2, 8)` densely and lands near a
/// rounding boundary often, which is where the last bit is hardest.
fn similar() -> impl Iterator<Item = [f128; 2]> {
    let leg = |i| f128::from_bits(1.0_f128.to_bits() | (common128::mix128(i) & FRACTION));
    (0..SAMPLE_COUNT).map(move |i| [leg(2 * i), leg(2 * i + 1)])
}

/// Representation-uniform pairs: scaling, the `dn > 56` early-out, and the
/// ∞/NaN/zero specials.
fn wide() -> impl Iterator<Item = [f128; 2]> {
    let leg = |i| f128::from_bits(common128::mix128(i));
    (0..SAMPLE_COUNT).map(move |i| [leg(2 * i), leg(2 * i + 1)])
}

/// Subnormals and the normal-subnormal transition, where the result grid is the
/// fixed 2^-16494 quantum instead of the candidate's own binade.
fn tiny() -> impl Iterator<Item = [f128; 2]> {
    let leg = |i| f128::from_bits(common128::mix128(i) >> 15);
    (0..SAMPLE_COUNT).map(move |i| [leg(2 * i), leg(2 * i + 1)])
}

/// The top four binades, straddling the overflow boundary.
fn huge() -> impl Iterator<Item = [f128; 2]> {
    let leg = |i| {
        let bits = common128::mix128(i);
        f128::from_bits((0x7ffb + (bits >> 126)) << 112 | (bits & FRACTION))
    };
    (0..SAMPLE_COUNT).map(move |i| [leg(2 * i), leg(2 * i + 1)])
}

/// Exponent gaps straddling the `dn > 56` early-out, where returning `|a|`
/// stops being the correctly rounded answer.
fn gaps() -> impl Iterator<Item = [f128; 2]> {
    (0..SAMPLE_COUNT).map(|i| {
        let bits = common128::mix128(i);
        let exponent = 0x2000 + (bits >> 116) % 0x1000;
        let gap = 50 + (bits >> 100) % 11;
        [
            f128::from_bits(exponent << 112 | (bits & FRACTION)),
            f128::from_bits((exponent - gap) << 112 | (common128::mix128(!i) & FRACTION)),
        ]
    })
}

/// Exact triples, the overflow edge, and the ∞/NaN/zero corners.
fn edges() -> impl Iterator<Item = [f128; 2]> {
    [
        [f128::MAX, f128::MAX],
        [f128::MAX, 0.0],
        [f128::MAX, f128::MIN_POSITIVE],
        [f128::from_bits(1), f128::from_bits(1)],
        [f128::from_bits(3), f128::from_bits(4)],
        [f128::from_bits(FRACTION), f128::from_bits(FRACTION)],
        [f128::MIN_POSITIVE, f128::from_bits(1)],
        // Tightest inputs for the `dn > 56` early-out: |a| an exact power of
        // two, |b| the largest mantissa 57 binades down, so that
        // √(a² + b²) − |a| misses ½ ulp(|a|) by a factor of only 1 − 2⁻¹¹².
        [
            f128::from_bits(0x3fff << 112),
            f128::from_bits((0x3fff - 57) << 112 | FRACTION),
        ],
        [
            f128::from_bits(0x3fff << 112),
            f128::from_bits((0x3fff - 56) << 112 | FRACTION),
        ],
        [f128::MIN_POSITIVE, f128::from_bits(FRACTION >> 57)],
        [3.0, 4.0],
        [-3.0, -4.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [-0.0, -0.0],
        [f128::INFINITY, f128::NAN],
        [f128::NAN, f128::NEG_INFINITY],
        [f128::NAN, 1.0],
        [f128::NAN, f128::NAN],
    ]
    .into_iter()
}

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("hypotq.wc", common128::parse_f128_pair).count(),
        11_642
    );
}

#[test]
fn test_hypotq() {
    common128::test_bivariate_cases_f128(
        metallic::hypotq,
        core_math::hypotq,
        similar()
            .chain(wide())
            .chain(tiny())
            .chain(huge())
            .chain(gaps())
            .chain(edges()),
    );
}

#[test]
fn test_hypotq_special() {
    assert!(metallic::hypotq(3.0, -4.0).is(&5.0));
    assert!(metallic::hypotq(-1.0, 0.0).is(&1.0));
    assert!(metallic::hypotq(-0.0, -0.0).is(&0.0));
    assert!(metallic::hypotq(f128::NEG_INFINITY, f128::NAN).is(&f128::INFINITY));
    assert!(metallic::hypotq(f128::NAN, f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::hypotq(f128::NAN, 1.0).is(&f128::NAN));
    assert!(metallic::hypotq(f128::MAX, f128::MAX).is(&f128::INFINITY));
    assert!(metallic::hypotq(f128::from_bits(3), f128::from_bits(4)).is(&f128::from_bits(5)));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_hypotq_worst_cases() {
    common128::test_worst_bivariate_f128("hypot", metallic::hypotq, core_math::hypotq);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_hypotq_vs_mpfr() {
    use rug::float::Round::Nearest;

    common128::mpfr_sweep_bivariate_f128(
        metallic::hypotq,
        |x, y| metallic::f128_mpfr::cr_binop(x, y, |a, b| a.hypot_round(b, Nearest)),
        |i| {
            [
                f128::from_bits(common128::mix128(2 * i)),
                f128::from_bits(common128::mix128(2 * i + 1)),
            ]
        },
        500_000,
    );
}
