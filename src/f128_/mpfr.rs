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
