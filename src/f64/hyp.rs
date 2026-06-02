use super::double::{fast_ldexp, Sum};
use super::exp::{exp_dd, exp_dd_fast};

/// Combine `(m, q)` — where `eˣ = 2`<sup>`q`</sup>` · m` for `x ≥ 0` — into the
/// mantissa `m ± 2⁻²q/m` so that `½(eˣ ± e⁻ˣ) = 2`<sup>`q−1`</sup>` · mantissa`.
///
/// `t = 2⁻²q/m ≈ e⁻ˣ` relative to `eˣ`; `exp2i(-2q)` underflows to 0 once the term
/// is negligible, so no explicit cutoff is needed.
#[inline]
fn combine(m: Sum, q: i64, add: bool) -> Sum {
    let t = m.recip() * crate::exp2i(-2 * q);
    if add {
        m + t
    } else {
        m + Sum {
            high: -t.high,
            low: -t.low,
        }
    }
}

/// Ziv gate for the hyperbolic fast path, as an absolute bound on the mantissa.
///
/// The fast `eˣ` mantissa is ≈2⁻⁶⁸ relative, and `m ± 2⁻²q/m` keeps that, so the
/// mantissa is good to ≈2⁻⁶⁷ absolute; `2⁻⁶²` keeps a ~30× margin.  The gate is on
/// the mantissa (∈ (0, 2]); for `sinh` it falls back automatically when `m − t`
/// cancels to a value too small for `2⁻⁶²` to resolve (only `|x| ≲ 2⁻¹⁰`).
const HYP_ZIV_EPS: f64 = 2.168_404_344_971_009e-19; // 2^-62

/// Hyperbolic cosine
#[must_use]
#[inline]
pub fn cosh(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }

    let x = x.abs();

    // `ln(2·f64::MAX)`: above this `cosh = eˣ/2` overflows.
    if x > 710.475_860_073_944 {
        return f64::INFINITY;
    }

    // cosh(x) = ½(eˣ + e⁻ˣ) = 2^(q−1)·(m + 2⁻²q/m); the sum never cancels.  Fast
    // path: lean `eˣ` mantissa accepted by a Ziv test, else the accurate one.
    let (m, q) = exp_dd_fast(x);
    let mantissa = combine(m, q, true);
    let lo = mantissa.high + (mantissa.low - HYP_ZIV_EPS);
    let hi = mantissa.high + (mantissa.low + HYP_ZIV_EPS);
    if lo == hi {
        return fast_ldexp(lo, q - 1);
    }

    let (m, q) = exp_dd(x);
    let mantissa = combine(m, q, true);
    fast_ldexp(mantissa.high + mantissa.low, q - 1)
}

/// Hyperbolic sine
#[must_use]
#[inline]
pub fn sinh(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }

    let s = x.abs();

    if s > 710.475_860_073_944 {
        return f64::INFINITY.copysign(x);
    }

    // For |x| ≤ 2⁻²⁶, sinh(x) = x + x³/6 + … rounds to exactly x.
    if s < 1.490_116_119_384_765_6e-8 {
        return x;
    }

    // sinh(x) = ½(eˣ − e⁻ˣ) = 2^(q−1)·(m − 2⁻²q/m).  The double-double subtraction
    // `m − 2⁻²q/m` captures the cancellation exactly (2Sum), so no separate
    // small-argument polynomial is needed above the 2⁻²⁶ threshold.  Fast path
    // with a Ziv test, as in `cosh`.
    let (m, q) = exp_dd_fast(s);
    let mantissa = combine(m, q, false);
    let lo = mantissa.high + (mantissa.low - HYP_ZIV_EPS);
    let hi = mantissa.high + (mantissa.low + HYP_ZIV_EPS);
    if lo == hi {
        return fast_ldexp(lo, q - 1).copysign(x);
    }

    let (m, q) = exp_dd(s);
    let mantissa = combine(m, q, false);
    fast_ldexp(mantissa.high + mantissa.low, q - 1).copysign(x)
}

/// Hyperbolic tangent
#[must_use]
#[inline]
pub fn tanh(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }

    let s = x.abs();

    // For x ≳ 19, tanh(x) = 1 − 2e⁻²ˣ rounds to exactly 1 (and this keeps e²ˣ from
    // overflowing below).
    if s >= 20.0 {
        return 1.0_f64.copysign(x);
    }

    // For |x| ≤ 2⁻²⁷, tanh(x) = x − x³/3 + … rounds to exactly x.  (The cubic term
    // is twice sinh's, so the threshold is half a binade smaller.)
    if s < 7.450_580_596_923_828e-9 {
        return x;
    }

    // tanh(x) = expm1(2x) / (expm1(2x) + 2).  Form expm1(2x) = 2^q·m − 1 as a
    // double-double, then the quotient in double-double.
    let (m, q) = exp_dd(2.0 * s);
    let t = m * crate::exp2i(q)
        + Sum {
            high: -1.0,
            low: 0.0,
        };
    let result = t
        * (t + Sum {
            high: 2.0,
            low: 0.0,
        })
        .recip();
    (result.high + result.low).copysign(x)
}
