use crate::f64::double::{fast_ldexp, round_general, DoubleDouble};
use crate::f64::pow::{log2_dd, log2_fast_path, poly_dd, EXP2_CE, EXP2_FAST};
use core::cmp::Ordering;
use core::num::FpCategory;

/// `2^e` correctly rounded to `f32`, taking a double-double exponent
///
/// Splits `e = n + h` with `n = round(e)` and `|h| ≤ ½`, evaluates `2ʰ` in
/// double-double via [`EXP2_CE`], scales by `2ⁿ` with [`fast_ldexp`], and rounds
/// the double-double with [`round_general`] (round-to-odd, subnormal-safe).  The
/// argument carries enough precision that the round is correct for [`powf`].
#[inline]
fn exp2_dd(e: DoubleDouble) -> f32 {
    if e.high > 130.0 {
        return f32::INFINITY;
    }
    if e.high < -160.0 {
        return 0.0;
    }

    let n = e.high.round_ties_even();
    let h = DoubleDouble::from_sum(e.high - n, e.low);
    let m = poly_dd(h, &EXP2_CE);

    // SAFETY: `-160 ≤ e.high ≤ 130` bounds `n`, so the scaling stays in range.
    let n = unsafe { n.to_int_unchecked() };

    round_general(DoubleDouble {
        high: fast_ldexp(m.high, n),
        low: fast_ldexp(m.low, n),
    })
}

/// `xʸ` for finite positive `x ≠ 1`, correctly rounded to `f32`
///
/// Fast path: `2^(y·log₂x)` entirely in `f64`.  The fast result is good to
/// ≈2⁻⁴⁵ relative, so unless its discarded low 29 bits land within `0x2000` of
/// the round-to-nearest midpoint — or it falls outside the normal `f32` range,
/// where the bit test does not apply — the single rounding is correct.  The few
/// ambiguous inputs (and the subnormal / near-overflow ends) take the
/// double-double [`log2_dd`]→`×y`→[`exp2_dd`] path.
#[inline]
fn powf_core(x: f64, y: f64) -> f32 {
    let e = log2_fast_path(x) * y;

    if e > 130.0 {
        return f32::INFINITY;
    }
    if e < -160.0 {
        return 0.0;
    }

    let n = e.round_ties_even();

    #[allow(clippy::cast_possible_truncation)]
    let r = fast_ldexp(crate::poly(e - n, &EXP2_FAST), n as i64);

    if r >= f64::from(f32::MIN_POSITIVE)
        && r < crate::exp2i(127)
        && (((r.to_bits() & 0x1FFF_FFFF) as i64) - 0x1000_0000).abs() > 0x2000
    {
        return r as f32;
    }

    exp2_dd(log2_dd(x) * y)
}

/// Raise to a floating-point power
#[must_use]
#[inline]
pub fn powf(x: f32, y: f32) -> f32 {
    #[inline]
    fn magnitude(x: f32, y: f32) -> f32 {
        match x.classify() {
            FpCategory::Nan => f32::NAN,
            FpCategory::Infinite => match y.partial_cmp(&0.0) {
                Some(Ordering::Greater) => f32::INFINITY,
                Some(Ordering::Less) => 0.0,
                Some(Ordering::Equal) => 1.0,
                None => f32::NAN,
            },
            FpCategory::Zero => match y.partial_cmp(&0.0) {
                Some(Ordering::Greater) => 0.0,
                Some(Ordering::Less) => f32::INFINITY,
                Some(Ordering::Equal) => 1.0,
                None => f32::NAN,
            },
            _ => match x {
                1.0 => 1.0,
                x if x.is_sign_negative() => f32::NAN,
                // xʸ = 2^(y·log₂x): a fast f64 pass with a Ziv gate, falling
                // back to double-double where the `×y` amplification of
                // `log₂x`'s error would otherwise cross an f32 rounding bound.
                _ => powf_core(x.into(), f64::from(y)),
            },
        }
    }

    #[inline]
    fn is_integer(x: f32) -> bool {
        x.trunc().eq(&x)
    }

    if y == 0.0 {
        return 1.0;
    }

    if x.is_sign_negative() && is_integer(y) {
        let sign = if is_integer(0.5 * y) { 1.0 } else { -1.0 };
        return sign * magnitude(-x, y);
    }

    magnitude(x, y)
}
