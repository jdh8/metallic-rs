use super::double::{DoubleDouble, fast_ldexp, sqrt_dd};
use super::exp::{exp_dd, exp_two_level_fast};
use super::{ln_fast, ln_fast_scaled};
use core::cmp::Ordering;

/// `1` as a double-double.
const ONE: DoubleDouble = DoubleDouble {
    high: 1.0,
    low: 0.0,
};

/// Lean leading `ln(s.high) + s.low/s.high` for the inverse-hyperbolic fast leg,
/// using the fast [`ln_fast`] kernel (<2⁻⁶⁶ absolute).  The linear correction
/// `s.low/s.high ≈ 2⁻⁵²` is all that survives of `ln(1 + s.low/s.high)`; on a Ziv
/// straddle the sound 128-bit [`super::dint::ln_dd_scaled`] takes over.
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
/// `ln_fast` differs from the accurate `ln_dd` by <2⁻⁶⁶ *absolute* whatever the
/// result's magnitude — its exact-`z` reduction commits no error, so only the
/// plain-`f64` `ln(1+z)` tail carries the lean leg's error (see `LN_ZIV_EPS` in
/// `log.rs`) — and `½·ln` (atanh) only halves it.  `2⁻⁶³` keeps an 8× margin.
/// Being absolute, the gate
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

    // Accurate leg: correctly-rounded `scale·ln(u)` via the 128-bit `dint` log of
    // the exact two-word `u`.  The double-double `ln_sum` (≈2⁻⁸⁷) mis-rounds the
    // hard-to-round ties this gate straddles; `dint`'s sound `to_f64` resolves them.
    let k = if scale == 0.5 { -1 } else { 0 };
    super::dint::ln_dd_scaled(u.high, u.low, k)
}

/// Correctly-rounded `ln(x + c)` for the `acosh`/`asinh` argument, where
/// `c = √(x² ∓ 1)` is a double-double and `x > 0`.
///
/// The fast leg gates the lean `ln_sum_fast(x + c)`; on a straddle the accurate
/// leg feeds *four* words — `x` and a triple-word `√d` — to the 128-bit `dint`
/// log.  The double-double sqrt (≈2⁻¹⁰⁵, ≈2⁻⁵⁷ ulp through `ln`) is one bit shy
/// of the corpus's hardest ties (≈2⁻⁶²), so a third sqrt word `corr` is refined
/// in and carried alongside `c.high`/`c.low`.
#[inline]
fn ln_sqrt_rounded(x: f64, c: DoubleDouble, d: DoubleDouble) -> f64 {
    let u = c + DoubleDouble { high: x, low: 0.0 };
    let DoubleDouble { high, low } = ln_sum_fast(u);
    let lo = high + (low - IHYP_ZIV_EPS);
    let hi = high + (low + IHYP_ZIV_EPS);
    if lo == hi {
        return lo;
    }

    // Refine `c = √d` with a third word so the argument reaches past the
    // double-double sqrt's ≈2⁻¹⁰⁵ (≈2⁻⁵⁷ ulp), enough for the hardest ties
    // (≈2⁻⁶²).  `corr = (d − c²)/(2c)`: the residual `d − c²` sits ≈106 bits below
    // the argument — below double-double range — so it is extracted by *exact*
    // cancellation (`from_product` for `c.high²` and the cross term, 2Sum chains
    // keeping every residual), not the flat `c·c` (which rounds back to `d`).
    let p = DoubleDouble::from_product(c.high, c.high);
    let diff = DoubleDouble::from_sum(d.high, -p.high) + DoubleDouble::from_sum(d.low, -p.low);
    let cr = DoubleDouble::from_product(c.high, c.low);
    let diff = diff
        + DoubleDouble {
            high: -2.0 * cr.high,
            low: -2.0 * cr.low,
        };
    // `(diff.high + diff.low) − c.low²`, the `c.low²` folded in with an exact FMA.
    let resid = crate::fma(-c.low, c.low, diff.high + diff.low);
    let corr = resid * (0.5 / c.high);
    super::dint::ln_quad_scaled(x, c.high, c.low, corr, 0)
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
    // Fast leg: `ln(2s)` straight from the scaled reduction (the `×2` folds into
    // the exponent, so no `+ LN2` double-double add), then fold `correction` into
    // the low word.  Since `s > 2²⁷`, the result is `≥ ln(2²⁸) ≈ 19` while
    // `|correction| = 0.25/s² ≤ 2⁻⁵⁶`, far below `ulp(high) ≥ 2⁻⁴⁸`, so it only
    // perturbs the low word — a single `f64` add instead of a second
    // double-double add.
    let DoubleDouble { high, low } = ln_fast_scaled(s, 1);
    let low = low + correction;
    let lo = high + (low - IHYP_ZIV_EPS);
    let hi = high + (low + IHYP_ZIV_EPS);
    if lo == hi {
        return lo;
    }

    // Accurate leg: correctly-rounded `ln(2s) + correction` via the 128-bit `dint`
    // log — the sound counterpart of the double-double `ln_dd(s) + LN2 + correction`.
    super::dint::ln_2s_corrected_accurate(s, correction)
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

    // `ln(2·f64::MAX)` rounded up to the first overflowing input: at and above
    // this `cosh = eˣ/2` rounds to ∞ (`0x1.633ce8fb9f87ep+9`; the value just
    // below it, `…87dp+9`, still rounds to a finite ≈`f64::MAX`).  Must be `>=`,
    // not `>`: the boundary input itself overflows, and letting it fall through
    // to the `eˣ` path yields `NaN` from the `2^q` reconstruction.
    if x >= 710.475_860_073_944 {
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

    // See `cosh`: `>=`, not `>` — the boundary input `0x1.633ce8fb9f87ep+9`
    // overflows to ∞, and the `eˣ` path would otherwise return `NaN` there.
    if s >= 710.475_860_073_944 {
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
        let d = DoubleDouble::from_product(s, s) + ONE;
        ln_sqrt_rounded(s, sqrt_dd(d), d)
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

    let d = DoubleDouble::from_product(x, x)
        + DoubleDouble {
            high: -1.0,
            low: 0.0,
        };
    ln_sqrt_rounded(x, sqrt_dd(d), d)
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
    let tl = crate::fast_mul_add(
        crate::fast_mul_add(
            ph,
            crate::fast_mul_add(-ql, iqh, crate::fma(-qh, iqh, 1.0)),
            pl,
        ),
        iqh,
        crate::fma(ph, iqh, -th),
    );
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
