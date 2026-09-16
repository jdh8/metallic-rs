//! `roundq` is exact bit manipulation: the nearest integer, ties away from
//! zero.  The gate is the definition — an independent integer reconstruction
//! of the same value on every sampled operand, the seams where the answer
//! changes binade, and the MPFR sweep.
use crate::common128;
use metallic::roundq;

const SAMPLES: u64 = 200_000;

/// Explicitly stored significand bits.
const EXP_SHIFT: u32 = 112;
const SIGN: u128 = 1 << 127;
const MANTISSA: u128 = (1 << EXP_SHIFT) - 1;

/// The nearest integer of a finite `x`, built from its integer significand
/// rather than from `roundq`'s add-and-mask.
fn reference(x: f128) -> f128 {
    let bits = x.to_bits();
    let exponent = ((bits >> EXP_SHIFT) & 0x7fff) as i32 - 16383;

    if exponent >= EXP_SHIFT as i32 {
        return x;
    }
    let magnitude = if exponent < 0 {
        // `|x| < 1`: the fraction is the whole value, so only `[0.5, 1)` lifts.
        f128::from(u8::from(exponent == -1))
    } else {
        // Subnormals never reach here (their exponent is below zero), so the
        // significand carries its implicit bit and `q + 1 ≤ 2^113` is exact.
        let significand = bits & MANTISSA | 1 << EXP_SHIFT;
        let shift = EXP_SHIFT - exponent as u32;
        let quotient = significand >> shift;
        let half = 1u128 << (shift - 1);

        (quotient + u128::from(significand & (half + half - 1) >= half)) as f128
    };
    f128::from_bits(bits & SIGN | magnitude.to_bits())
}

#[test]
fn test_roundq() {
    for i in 0..SAMPLES {
        let x = f128::from_bits(common128::mix128(i));
        assert!(
            roundq(x).to_bits() == reference(x).to_bits() || x.is_nan(),
            "roundq({x:?})"
        );
    }
}

/// Every operand of exponent 112 and up is already an integer, and so is every
/// nonfinite one: all of them come back bit for bit.
#[test]
fn integral_and_nonfinite() {
    for i in 0..SAMPLES / 10 {
        let bits = common128::mix128(i);

        for e in [0x3fff + EXP_SHIFT as u128, 0x5000, 0x7ffe, 0x7fff] {
            let x = f128::from_bits(bits & SIGN | e << EXP_SHIFT | bits & MANTISSA);
            assert!(
                roundq(x).to_bits() == x.to_bits() || x.is_nan(),
                "roundq({x:?})"
            );
        }
    }
    assert!(roundq(f128::NAN).is_nan());
    assert_eq!(roundq(f128::INFINITY), f128::INFINITY);
    assert_eq!(roundq(f128::NEG_INFINITY), f128::NEG_INFINITY);
    assert_eq!(roundq(f128::MAX), f128::MAX);
}

/// Ties go away from zero, and the value just under a tie does not.
#[test]
fn halfway_cases() {
    let down = f128::from_bits(0.5_f128.to_bits() - 1); // the largest `< 0.5`

    assert_eq!(roundq(0.5_f128), 1.0);
    assert_eq!(roundq(-0.5_f128), -1.0);
    assert_eq!(roundq(down).to_bits(), 0);
    assert_eq!(roundq(-down).to_bits(), SIGN);

    for k in 0..=111 {
        let n = (1u128 << k) as f128; // an integer whose successor gap is < 1
        assert_eq!(roundq(n + 0.5), n + 1.0, "roundq(2^{k} + 0.5)");
        assert_eq!(roundq(-n - 0.5), -n - 1.0, "roundq(-2^{k} - 0.5)");
        assert_eq!(roundq(n + 0.25), n, "roundq(2^{k} + 0.25)");
        assert_eq!(roundq(n - 0.5), n, "roundq(2^{k} - 0.5)");
    }
}

/// Zeros and every subnormal collapse to a zero of the same sign, and the
/// binade below one splits at `±0.5`.
#[test]
fn below_one() {
    assert_eq!(roundq(0.0_f128).to_bits(), 0);
    assert_eq!(roundq(-0.0_f128).to_bits(), SIGN);

    for k in -16494..0 {
        let x = metallic::ldexpq(1.0, k);
        let want: f128 = if k == -1 { 1.0 } else { 0.0 };
        assert_eq!(roundq(x), want, "roundq(2^{k})");
        assert_eq!(
            roundq(-x).to_bits(),
            SIGN | want.to_bits(),
            "roundq(-2^{k})"
        );
        // And the top of each binade, just under the next power of two.
        let top = f128::from_bits(metallic::ldexpq(1.0, k + 1).to_bits() - 1);
        assert_eq!(roundq(top), want, "roundq(2^{} - ulp)", k + 1);
    }
}

#[cfg(feature = "mpfr")]
#[test]
fn test_roundq_vs_mpfr() {
    use core::cmp::Ordering;

    // MPFR's `round` is ties-away like ours, and it is exact at precision 113
    // (the result needs no more bits than the operand), so the ternary value
    // is `Equal` and `cr_unop` has nothing left to round.
    for i in 0..SAMPLES {
        let x = f128::from_bits(common128::mix128(i));
        let want = metallic::f128_mpfr::cr_unop(x, |y| {
            y.round_mut();
            Ordering::Equal
        });
        let got = roundq(x);
        assert!(
            got.to_bits() == want.to_bits() || (got.is_nan() && want.is_nan()),
            "roundq({x:?}) = {got:?} != {want:?}"
        );
    }
}
