//! MPFR bridges for binary128 tests and soundness checks.

use core::cmp::Ordering;
use rug::{Float, float::Round};

const PRECISION: u32 = f128::MANTISSA_DIGITS;

/// Convert an `f128` to an exact MPFR value.
#[must_use]
pub fn to_mpfr(x: f128) -> Float {
    Float::with_val(PRECISION, x)
}

/// Round an MPFR value to binary128 using round-to-nearest, ties-to-even.
#[must_use]
pub fn from_mpfr(x: &Float) -> f128 {
    x.to_f128_round(Round::Nearest)
}

/// Evaluate a unary MPFR operation with the exact binary128 rounding recipe.
///
/// The callback must mutate its 113-bit operand in place and return MPFR's
/// ternary rounding result.
pub fn cr_unop(x: f128, op: impl FnOnce(&mut Float) -> Ordering) -> f128 {
    let mut y = to_mpfr(x);
    let rounding = op(&mut y);
    y.subnormalize_ieee_round(rounding, Round::Nearest);
    from_mpfr(&y)
}

/// Evaluate a binary MPFR operation with the exact binary128 rounding recipe.
///
/// The callback must mutate its 113-bit left operand in place and return MPFR's
/// ternary rounding result.
pub fn cr_binop(x: f128, y: f128, op: impl FnOnce(&mut Float, &Float) -> Ordering) -> f128 {
    let mut z = to_mpfr(x);
    let rounding = op(&mut z, &to_mpfr(y));
    z.subnormalize_ieee_round(rounding, Round::Nearest);
    from_mpfr(&z)
}

/// Evaluate a ternary MPFR operation with the exact binary128 rounding recipe.
///
/// The callback must mutate its 113-bit first operand in place and return
/// MPFR's ternary rounding result.
pub fn cr_terop(
    x: f128,
    y: f128,
    z: f128,
    op: impl FnOnce(&mut Float, &Float, &Float) -> Ordering,
) -> f128 {
    let mut w = to_mpfr(x);
    let rounding = op(&mut w, &to_mpfr(y), &to_mpfr(z));
    w.subnormalize_ieee_round(rounding, Round::Nearest);
    from_mpfr(&w)
}

/// `(1+x)^y` with an exact MPFR base and a precision-113 power operation.
///
/// MPFR exposes compound only for integer exponents. Most pairs are decided
/// cheaply by directed bounds on exp(y*log1p(x)); if those bounds round
/// differently (including exact midpoints), form 1+x with enough bits to be
/// exact, then ask MPFR to round pow directly to 113 bits. Its ternary result
/// drives IEEE subnormalization, avoiding a second rounding at underflow.
pub fn cr_compound(x: f128, y: f128) -> f128 {
    use rug::ops::{MulAssignRound, Pow};
    let snan = |v: f128| v.is_nan() && v.to_bits() & (1 << 111) == 0;
    if snan(x) || snan(y) || x < -1.0 {
        return f128::NAN;
    }
    if x == 0.0 || y == 0.0 {
        return 1.0;
    }
    if x.is_nan() || y.is_nan() {
        return f128::NAN;
    }
    if x == -1.0 {
        return if y > 0.0 { 0.0 } else { f128::INFINITY };
    }
    if x == f128::INFINITY || y.is_infinite() {
        return if (x > 0.0) == (y > 0.0) {
            f128::INFINITY
        } else {
            0.0
        };
    }
    let mut lo = Float::with_val(256, x);
    let mut hi = lo.clone();
    lo.ln_1p_round(Round::Down);
    hi.ln_1p_round(Round::Up);
    if y < 0.0 {
        core::mem::swap(&mut lo, &mut hi);
    }
    lo.mul_assign_round(y, Round::Down);
    hi.mul_assign_round(y, Round::Up);
    lo.exp_round(Round::Down);
    hi.exp_round(Round::Up);
    let lower = from_mpfr(&lo);
    if lower.to_bits() == from_mpfr(&hi).to_bits() {
        return lower;
    }

    let e = ((x.to_bits() >> 112) & 0x7fff) as i32 - 16383;
    let precision = if e < 0 {
        114 - e.max(-16382)
    } else {
        (e + 2).max(114)
    } as u32;
    let base = Float::with_val(precision, x) + 1u32;
    let (mut value, direction) =
        Float::with_val_round(113, (&base).pow(to_mpfr(y)), Round::Nearest);
    value.subnormalize_ieee_round(direction, Round::Nearest);
    from_mpfr(&value)
}
