//! `ldexpq` is exact scaling by a power of two.  There is no binary128 `libm`
//! lane to diff against, so the default gate is the definition itself — inside
//! the normal range the result is the operand with `n` added to its exponent
//! field — plus the subnormal ladder, where a single bit walks up from
//! `2^-16494` into the normal range.  The MPFR sweep covers the rest.
use crate::common128;
use metallic::{frexpq, ldexpq};

const SAMPLES: u64 = 20_000;

/// Explicitly stored significand bits.
const EXP_SHIFT: u32 = 112;
const SIGN: u128 = 1 << 127;
const MANTISSA: u128 = (1 << EXP_SHIFT) - 1;

/// A deterministic normal operand whose biased exponent is `e`.
fn normal(i: u64, e: u128) -> f128 {
    let bits = common128::mix128(i);
    f128::from_bits(bits & SIGN | e << EXP_SHIFT | bits & MANTISSA)
}

/// Scaling a normal operand to a normal result only adds to the exponent field.
#[test]
fn exponent_field() {
    for i in 0..SAMPLES {
        let e = 1 + u128::from(common128::mix128(i) >> 64) % 0x7ffe;
        let x = normal(i, e);

        for n in (1 - e as i32..0x7fff - e as i32).step_by(97) {
            let scaled = ldexpq(x, n);
            assert_eq!(
                scaled.to_bits(),
                x.to_bits().wrapping_add((n as i128 as u128) << EXP_SHIFT),
                "ldexpq({x:?}, {n})"
            );
            // And it is undone exactly.
            assert_eq!(ldexpq(scaled, -n).to_bits(), x.to_bits());
        }
    }
}

/// One bit walking out of the subnormals: `2^-16494 · 2^k` is `from_bits(1 << k)`
/// for every `k` the bit pattern still spells, the implicit bit included.
#[test]
fn subnormal_ladder() {
    let tiny = f128::from_bits(1);

    for k in 0..=113 {
        assert_eq!(
            ldexpq(tiny, k).to_bits(),
            1u128 << k,
            "ldexpq(2^-16494, {k})"
        );
        assert_eq!(ldexpq(-tiny, k).to_bits(), SIGN | 1u128 << k);
    }
}

/// Rounding happens at most once, and only on the way into the subnormals.
#[test]
fn underflow_and_overflow() {
    // Halfway between zero and the smallest subnormal rounds to even, i.e. zero.
    assert_eq!(ldexpq(f128::from_bits(1), -1).to_bits(), 0);
    assert_eq!(ldexpq(f128::from_bits(3), -1).to_bits(), 2);
    assert_eq!(ldexpq(f128::from_bits(SIGN | 1), -1).to_bits(), SIGN);

    // `f128::MAX` is just under `2^16384`, so underflowing it all the way to a
    // zero takes more than one exponent range, as overflowing `MIN_POSITIVE` does.
    for n in [-32_879, -50_000, -100_000, i32::MIN] {
        assert_eq!(ldexpq(f128::MAX, n).to_bits(), 0, "ldexpq(MAX, {n})");
        assert_eq!(ldexpq(-f128::MAX, n).to_bits(), SIGN);
    }
    for n in [32_767, 50_000, 100_000, i32::MAX] {
        assert_eq!(
            ldexpq(f128::MIN_POSITIVE, n),
            f128::INFINITY,
            "ldexpq(MIN_POSITIVE, {n})"
        );
        assert_eq!(ldexpq(-f128::MIN_POSITIVE, n), f128::NEG_INFINITY);
    }
}

#[test]
fn nonfinite() {
    for n in (-100_000..=100_000).step_by(997) {
        assert_eq!(ldexpq(f128::INFINITY, n), f128::INFINITY);
        assert_eq!(ldexpq(f128::NEG_INFINITY, n), f128::NEG_INFINITY);
        assert!(ldexpq(f128::NAN, n).is_nan());
        assert_eq!(ldexpq(0.0_f128, n).to_bits(), 0);
        assert_eq!(ldexpq(-0.0_f128, n).to_bits(), SIGN);
    }
}

/// `frexpq` and `ldexpq` are inverses on every finite nonzero operand.
#[test]
fn round_trip() {
    for i in 0..SAMPLES {
        let x = f128::from_bits(common128::mix128(i));
        let (significand, exponent) = frexpq(x);

        if x.is_finite() && x != 0.0 {
            assert!((0.5..1.0).contains(&significand.abs()), "frexpq({x:?})");
        }
        assert_eq!(ldexpq(significand, exponent).to_bits(), x.to_bits());
    }
}

#[cfg(feature = "mpfr")]
#[test]
fn test_ldexpq_vs_mpfr() {
    use core::cmp::Ordering;

    // Shifting is exact in MPFR's unbounded exponent range, so the ternary
    // result is `Equal` and `cr_unop`'s IEEE subnormalization does all the
    // rounding — which is exactly `ldexpq`'s contract.
    for i in 0..SAMPLES {
        let x = f128::from_bits(common128::mix128(i));

        for n in (-17_000..=17_000).step_by(131) {
            let want = metallic::f128_mpfr::cr_unop(x, |y| {
                *y <<= n;
                Ordering::Equal
            });
            let got = ldexpq(x, n);
            assert!(
                got.to_bits() == want.to_bits() || (got.is_nan() && want.is_nan()),
                "ldexpq({x:?}, {n}) = {got:?} != {want:?}"
            );
        }
    }
}
