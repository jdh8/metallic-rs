//! `frexpq` splits a binary128 into a significand in `[0.5, 1)` and an
//! exponent.  Both halves are exact, so the gate is the pair of invariants:
//! the significand's range and the exact `ldexpq` round trip (which
//! [`super::ldexpq::round_trip`] also exercises from the other side).
use crate::common128;
use core::num::FpCategory;
use metallic::{frexpq, ldexpq};

const SAMPLES: u64 = 200_000;

#[test]
fn test_frexpq() {
    for i in 0..SAMPLES {
        let x = f128::from_bits(common128::mix128(i));
        let (significand, exponent) = frexpq(x);

        match x.classify() {
            FpCategory::Nan => assert!(significand.is_nan()),
            FpCategory::Infinite | FpCategory::Zero => {
                assert_eq!(significand.to_bits(), x.to_bits());
                assert_eq!(exponent, 0, "frexpq({x:?})");
            }
            _ => {
                assert!((0.5..1.0).contains(&significand.abs()), "frexpq({x:?})");
                assert_eq!(significand.is_sign_negative(), x.is_sign_negative());
                assert_eq!(ldexpq(significand, exponent).to_bits(), x.to_bits());
            }
        }
    }
}

/// The exponent is the one [`f128::MAX_EXP`] and [`f128::MIN_EXP`] name: every
/// power of two splits as `(0.5, k + 1)`, across the whole range including the
/// subnormals.
#[test]
fn powers_of_two() {
    for k in f128::MIN_EXP - f128::MANTISSA_DIGITS as i32..f128::MAX_EXP {
        let x = ldexpq(1.0, k);
        assert_eq!(frexpq(x), (0.5, k + 1), "frexpq(2^{k})");
        assert_eq!(frexpq(-x), (-0.5, k + 1), "frexpq(-2^{k})");
    }
}

#[test]
fn edges() {
    assert_eq!(
        frexpq(f128::MAX),
        (1.0 - f128::EPSILON / 2.0, f128::MAX_EXP)
    );
    assert_eq!(frexpq(f128::MIN_POSITIVE), (0.5, f128::MIN_EXP));
    assert_eq!(frexpq(f128::from_bits(1)), (0.5, -16493));
    assert_eq!(frexpq(0.0_f128).0.to_bits(), 0);
    assert_eq!(frexpq(-0.0_f128).0.to_bits(), 1 << 127);
}
