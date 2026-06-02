use super::double::{fast_ldexp, fast_sum, Sum};
use super::exp::{exp_dd, exp_dd_fast};
use super::ln_dd;
use core::cmp::Ordering;

/// `ln(2)` as a double-double (CORE-MATH's split, matching `log.rs`).
const LN2: Sum = Sum {
    high: 0.6931471805598903,
    low: 5.497923018708371e-14,
};

/// `1` as a double-double.
const ONE: Sum = Sum {
    high: 1.0,
    low: 0.0,
};

/// Square root of a non-negative double-double, refined by one Newton step
/// `h + (s − h²)/(2h)` to ≈2⁻¹⁰⁵ relative.  `s.high` must be strictly positive.
#[inline]
fn sqrt_dd(s: Sum) -> Sum {
    let h = s.high.sqrt();
    let h2 = Sum::from_product(h, h);
    let residual = (s.high - h2.high) + (s.low - h2.low);
    fast_sum(h, residual * (0.5 / h))
}

/// Natural logarithm of a positive double-double, as a double-double.
///
/// `ln(s) = ln(s.high) + ln(1 + s.low/s.high) ≈ ln_dd(s.high) + s.low/s.high`,
/// the linear term being all that survives since `s.low/s.high ≈ 2⁻⁵²`.
#[inline]
fn ln_sum(s: Sum) -> Sum {
    ln_dd(s.high)
        + Sum {
            high: s.low / s.high,
            low: 0.0,
        }
}

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

/// Inverse hyperbolic sine
///
/// `asinh(x) = ln(x + √(x² + 1))`, odd.  The log argument is carried as a
/// double-double and fed to [`ln_sum`]; for huge `|x|` (where `x²` would
/// overflow) it collapses to `ln(2·|x|) = ln|x| + ln 2`.
#[must_use]
#[inline]
pub fn asinh(x: f64) -> f64 {
    let s = x.abs();

    // ±∞ → ±∞ and NaN → NaN both equal `x`.
    if !s.is_finite() {
        return x;
    }

    // For |x| ≤ 2⁻²⁷, asinh(x) = x − x³/6 + … rounds to exactly x.
    if s < 7.450_580_596_923_828e-9 {
        return x;
    }

    let magnitude = if s > 1.0e150 {
        // `x²` would overflow; asinh(x) = ln(2·|x|), the `1/(2x)` correction being
        // below 2⁻¹⁰⁰ relative.
        let r = ln_dd(s) + LN2;
        r.high + r.low
    } else {
        let c = sqrt_dd(Sum::from_product(s, s) + ONE);
        let r = ln_sum(c + Sum { high: s, low: 0.0 });
        r.high + r.low
    };

    magnitude.copysign(x)
}

/// Inverse hyperbolic cosine
///
/// `acosh(x) = ln(x + √(x² − 1))` for `x ≥ 1`.  `x² − 1` is formed as a
/// double-double (exact, so the cancellation near `x = 1` is harmless) and the
/// log argument fed to [`ln_sum`]; huge `x` collapses to `ln(2x)`.
#[must_use]
#[inline]
pub fn acosh(x: f64) -> f64 {
    // `x < 1` (including −∞) is outside the domain; NaN propagates.
    if !(x >= 1.0) {
        return f64::NAN;
    }
    // acosh(1) = 0 exactly (and avoids 0/0 in `sqrt_dd`); +∞ → +∞ (avoids `ln_dd(∞)`).
    if x == 1.0 {
        return 0.0;
    }
    if x == f64::INFINITY {
        return x;
    }

    if x > 1.0e150 {
        // `x²` would overflow; acosh(x) = ln(2x).
        let r = ln_dd(x) + LN2;
        return r.high + r.low;
    }

    let c = sqrt_dd(
        Sum::from_product(x, x)
            + Sum {
                high: -1.0,
                low: 0.0,
            },
    );
    let r = ln_sum(c + Sum { high: x, low: 0.0 });
    r.high + r.low
}

/// Inverse hyperbolic tangent
///
/// `atanh(x) = ½·ln((1 + x)/(1 − x))` for `|x| < 1`, odd.  Both `1 ± x` are
/// formed exactly as double-doubles (2Sum) and divided in double-double, so the
/// log argument keeps full precision; [`ln_sum`] then gives the logarithm.
#[must_use]
#[inline]
pub fn atanh(x: f64) -> f64 {
    let s = x.abs();

    match s.partial_cmp(&1.0) {
        Some(Ordering::Less) => {
            // For |x| ≤ 2⁻²⁷, atanh(x) = x + x³/3 + … rounds to exactly x.
            if s < 7.450_580_596_923_828e-9 {
                return x;
            }

            // (1 + |x|)/(1 − |x|) in double-double, then ½·ln of it.
            let u = Sum::from_sum(1.0, s) * Sum::from_sum(1.0, -s).recip();
            let r = ln_sum(u) * 0.5;
            (r.high + r.low).copysign(x)
        }
        Some(Ordering::Equal) => f64::INFINITY.copysign(x),
        // |x| > 1 is outside the domain; NaN (the `None` case) propagates.
        _ => f64::NAN,
    }
}
