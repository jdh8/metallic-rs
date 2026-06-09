use super::double::{DoubleDouble, fast_ldexp, sqrt_dd};
use super::exp::{exp_dd, exp_two_level_fast};
use super::{ln_dd, ln_fast};
use core::cmp::Ordering;

/// `ln(2)` as a double-double (CORE-MATH's split, matching `log.rs`).
const LN2: DoubleDouble = DoubleDouble {
    high: 0.693_147_180_559_890_3,
    low: 5.497_923_018_708_371e-14,
};

/// `1` as a double-double.
const ONE: DoubleDouble = DoubleDouble {
    high: 1.0,
    low: 0.0,
};

/// Natural logarithm of a positive double-double, as a double-double.
///
/// `ln(s) = ln(s.high) + ln(1 + s.low/s.high) ≈ ln_dd(s.high) + s.low/s.high`,
/// the linear term being all that survives since `s.low/s.high ≈ 2⁻⁵²`.
#[inline]
fn ln_sum(s: DoubleDouble) -> DoubleDouble {
    ln_dd(s.high)
        + DoubleDouble {
            high: s.low / s.high,
            low: 0.0,
        }
}

/// Lean variant of [`ln_sum`] using the fast [`ln_fast`] kernel (≈2⁻⁶⁸ absolute
/// instead of ≈2⁻⁸⁷).  Only the leading `ln(s.high)` is leaner; the linear
/// correction `s.low/s.high` is identical, so it cancels in the Ziv comparison.
#[inline]
fn ln_sum_fast(s: DoubleDouble) -> DoubleDouble {
    ln_fast(s.high)
        + DoubleDouble {
            high: s.low / s.high,
            low: 0.0,
        }
}

/// Ziv gate for the inverse-hyperbolic fast path, as an absolute bound on the
/// result (`scale · ln(u)`).
///
/// `ln_fast` differs from the accurate `ln_dd` by ≈2⁻⁶⁸ *absolute* whatever the
/// result's magnitude — the `e·ln2 + L[i]` terms are double-double and shared, so
/// only the `ln(1+r)` tail (≤ 1/256) carries the lean kernel's error — and `½·ln`
/// (atanh) only halves it.  `2⁻⁶³` keeps a ~30× margin.  Being absolute, the gate
/// forces the accurate fallback only when `|result| ≲ 2⁻¹⁰`, where the lean kernel
/// cannot round correctly anyway.
const IHYP_ZIV_EPS: f64 = 1.084_202_172_485_504_4e-19; // 2^-63

/// Round `scale · ln(u)` for a positive double-double `u` via a two-step Ziv
/// test: the lean [`ln_sum_fast`] is accepted unless it straddles a rounding
/// boundary, in which case the accurate [`ln_sum`] resolves it.
#[inline]
fn ln_sum_rounded(u: DoubleDouble, scale: f64) -> f64 {
    let DoubleDouble { high, low } = ln_sum_fast(u) * scale;
    let lo = high + (low - IHYP_ZIV_EPS);
    let hi = high + (low + IHYP_ZIV_EPS);
    if lo == hi {
        return lo;
    }

    let r = ln_sum(u) * scale;
    r.high + r.low
}

/// `2²⁷`.  Above this magnitude `asinh`/`acosh` switch from `sqrt_dd(x² ± 1)` to
/// the sqrt-free asymptotic [`ln_2x_corrected`].
const LARGE_IHYP: f64 = 134_217_728.0;

/// Large-magnitude `asinh`/`acosh`: `ln(2·|x|) + correction`, where
/// `correction = ±0.25/x²` is `+1/(4x²)` for `asinh` and `−1/(4x²)` for `acosh`.
///
/// From `√(x² ± 1) = |x|·√(1 ± 1/x²)`, `asinh(x) = ln(2|x|) + 1/(4x²) − 5/(32x⁴) + …`
/// and `acosh(x) = ln(2x) − 1/(4x²) − 3/(32x⁴) − …`.  For `|x| ≥ 2²⁷` the dropped
/// `O(1/x⁴)` term is below 2⁻¹¹⁰, so the single `correction` makes the result
/// correctly rounded with no square root.  The lean `ln_fast` is accepted by the
/// absolute Ziv gate or deferred to the accurate `ln_dd`; `correction` is the same
/// in both legs, so it cancels in the comparison and the gate bounds only
/// `ln_fast`'s slip.  For `|x| > 1.34e154`, `x·x` overflows and `0.25/∞ = +0.0` —
/// the correct negligible value, subsuming the old `ln(2x)`-only branch.
#[inline]
fn ln_2x_corrected(s: f64, correction: f64) -> f64 {
    // Fast leg: `ln_fast(s) + LN2` as a double-double, then fold `correction` into
    // the low word.  Since `s > 2²⁷`, the result is `≥ ln(2²⁸) ≈ 19` while
    // `|correction| = 0.25/s² ≤ 2⁻⁵⁶`, far below `ulp(high) ≥ 2⁻⁴⁸`, so it only
    // perturbs the low word — a single `f64` add instead of a second
    // double-double add.
    let DoubleDouble { high, low } = ln_fast(s) + LN2;
    let low = low + correction;
    let lo = high + (low - IHYP_ZIV_EPS);
    let hi = high + (low + IHYP_ZIV_EPS);
    if lo == hi {
        return lo;
    }

    // Accurate leg unchanged: the full double-double `ln_dd(s) + LN2 + correction`.
    let r = ln_dd(s)
        + LN2
        + DoubleDouble {
            high: correction,
            low: 0.0,
        };
    r.high + r.low
}

/// Combine `(m, q)` — where `eˣ = 2`<sup>`q`</sup>` · m` for `x ≥ 0` — into the
/// mantissa `m ± 2⁻²q/m` so that `½(eˣ ± e⁻ˣ) = 2`<sup>`q−1`</sup>` · mantissa`.
///
/// `t = 2⁻²q/m ≈ e⁻ˣ` relative to `eˣ`; `exp2i(-2q)` underflows to 0 once the term
/// is negligible, so no explicit cutoff is needed.
#[inline]
fn combine(m: DoubleDouble, q: i64, add: bool) -> DoubleDouble {
    let t = m.recip() * crate::exp2i(-2 * q);
    if add {
        m + t
    } else {
        m + DoubleDouble {
            high: -t.high,
            low: -t.low,
        }
    }
}

/// Lean counterpart of [`combine`] for the fast leg: forms `t = 2⁻²q/m ≈ e⁻ˣ`
/// (relative to `eˣ`) with a single `f64` division instead of the double-double
/// reciprocal of [`combine`].
///
/// Used only when `q ≥ 8`.  There `t < 2⁻¹⁶`, so the `f64` reciprocal's relative
/// error contributes only `t·2⁻⁵³ ≲ 2⁻⁶⁹` absolutely — far inside the
/// [`HYP_ZIV_EPS`] gate — and `m − t` (sinh) cannot catastrophically cancel.  For
/// smaller `q` the term approaches `1`, where that error would breach the gate and
/// the cancellation needs full precision, so it defers to the dd [`combine`].
/// `exp2i(-2q)` underflows to `0` once `e⁻ˣ` is negligible, collapsing `t` to `0`
/// (the result is then `½eˣ`).
#[inline]
fn combine_fast(m: DoubleDouble, q: i64, add: bool) -> DoubleDouble {
    if q < 8 {
        return combine(m, q, add);
    }
    let t = crate::exp2i(-2 * q) / m.high;
    m + DoubleDouble {
        high: if add { t } else { -t },
        low: 0.0,
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
    let (m, q) = exp_two_level_fast(x);
    let mantissa = combine_fast(m, q, true);
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
    let (m, q) = exp_two_level_fast(s);
    let mantissa = combine_fast(m, q, false);
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
        + DoubleDouble {
            high: -1.0,
            low: 0.0,
        };
    let result = t
        * (t + DoubleDouble {
            high: 2.0,
            low: 0.0,
        })
        .recip();
    (result.high + result.low).copysign(x)
}

/// Inverse hyperbolic sine
///
/// `asinh(x) = ln(x + √(x² + 1))`, odd.  For `|x| ≤ 2²⁷` the log argument is
/// carried as a double-double and fed to [`ln_sum`]; for larger `|x|` it collapses
/// to the sqrt-free [`ln_2x_corrected`] (`ln(2|x|) + 1/(4x²)`), which also avoids
/// the `x²` overflow at the top of the range.
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

    let magnitude = if s > LARGE_IHYP {
        // asinh(x) = ln(2|x|) + 1/(4x²) − …, no square root.
        ln_2x_corrected(s, 0.25 / (s * s))
    } else {
        let c = sqrt_dd(DoubleDouble::from_product(s, s) + ONE);
        ln_sum_rounded(c + DoubleDouble { high: s, low: 0.0 }, 1.0)
    };

    magnitude.copysign(x)
}

/// Inverse hyperbolic cosine
///
/// `acosh(x) = ln(x + √(x² − 1))` for `x ≥ 1`.  For `x ≤ 2²⁷`, `x² − 1` is formed
/// as a double-double (exact, so the cancellation near `x = 1` is harmless) and the
/// log argument fed to [`ln_sum`]; larger `x` collapses to the sqrt-free
/// [`ln_2x_corrected`] (`ln(2x) − 1/(4x²)`), which also avoids the `x²` overflow at
/// the top of the range.
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

    if x > LARGE_IHYP {
        // acosh(x) = ln(2x) − 1/(4x²) − …, no square root.
        return ln_2x_corrected(x, -0.25 / (x * x));
    }

    let c = sqrt_dd(
        DoubleDouble::from_product(x, x)
            + DoubleDouble {
                high: -1.0,
                low: 0.0,
            },
    );
    ln_sum_rounded(c + DoubleDouble { high: x, low: 0.0 }, 1.0)
}

/// `(1 + s)/(1 − s)` as a double-double for `s ∈ (0, 1)`, formed with a single
/// `f64` division.
///
/// `1 ± s` are exact double-doubles (2Sum); one reciprocal `iqh = 1/qh` then
/// drives the standard double-double division correction (all remaining steps
/// FMAs), reaching ≈2⁻¹⁰⁶.  This is the atan "fuse the quotient" trick: it
/// replaces the double-double reciprocal *plus* double-double product of the
/// naive `from_sum(1, s) · from_sum(1, −s).recip()` with one division and a
/// short FMA chain.
#[inline]
fn ratio_1ps(s: f64) -> DoubleDouble {
    let DoubleDouble { high: ph, low: pl } = DoubleDouble::from_sum(1.0, s);
    let DoubleDouble { high: qh, low: ql } = DoubleDouble::from_sum(1.0, -s);

    let iqh = 1.0 / qh;
    let th = ph * iqh;
    // tl = (ph − th·qh − th·ql + pl)/qh, evaluated from `iqh` alone: the rounding
    // error of `th`, plus the reciprocal residual `(1 − qh·iqh)` and `−ql·iqh`
    // amplified by `ph`, all scaled by `iqh`.
    let tl =
        f64::mul_add(ph, iqh, -th) + (pl + ph * (f64::mul_add(-qh, iqh, 1.0) - ql * iqh)) * iqh;
    DoubleDouble { high: th, low: tl }
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
            let u = ratio_1ps(s);
            ln_sum_rounded(u, 0.5).copysign(x)
        }
        Some(Ordering::Equal) => f64::INFINITY.copysign(x),
        // |x| > 1 is outside the domain; NaN (the `None` case) propagates.
        _ => f64::NAN,
    }
}
