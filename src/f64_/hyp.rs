use super::double::{DoubleDouble, fast_ldexp, fast_sum, round_anchored, round_general64, sqrt_dd};
use super::exp::{exp_dd, exp_two_level_fast, exp_two_level_mantissa_accurate};
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

/// Correctly-rounded `atanh(s) = ½·ln(u)` for `u = (1 + s)/(1 − s)` and
/// `0 < s < 1`, via a two-step Ziv test.
///
/// The lean `½·[ln_sum_fast]` is accepted unless it straddles a rounding
/// boundary.  On a straddle the accurate leg is chosen by magnitude: for
/// `s < ATANH_SMALL` the result-anchored [`atanh_small_accurate`] (the dd `ln`
/// argument otherwise loses the small part to its leading `1`), otherwise the
/// sound 128-bit [`super::dint::ln_dd_scaled`].  Keeping the lean fast leg here
/// (rather than a series fast leg) holds the straddle rate to the baseline's
/// `≈0.5%`: the lean leg is `<2⁻⁶⁶` absolute, so its Ziv gate is far tighter than
/// a plain-`f64` series', avoiding heavy fallback into the degree-22 accurate
/// series over `atanh`'s wide small-`|x|` band.
#[inline]
fn atanh_rounded(u: DoubleDouble, s: f64) -> f64 {
    let DoubleDouble { high, low } = ln_sum_fast(u) * 0.5;
    let lo = high + (low - IHYP_ZIV_EPS);
    let hi = high + (low + IHYP_ZIV_EPS);
    if lo == hi {
        return lo;
    }

    if s < ATANH_SMALL {
        atanh_small_accurate(s)
    } else {
        super::dint::ln_dd_scaled(u.high, u.low, -1)
    }
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

    cosh_accurate(x)
}

/// Hard-to-round database for [`cosh_accurate`]: non-negative inputs whose
/// `cosh` lies within the double-double path's reach of an `f64` midpoint —
/// closer than it can resolve — mapped (by bit pattern of `|x|`) to their
/// correctly-rounded results.  These are the residuals metallic's accurate path
/// leaves on the `cosh.wc` corpus; the corpus enumerates every input ≥ 40
/// hard-to-round bits and the double-double error is far below that threshold
/// (no cancellation in `eˣ + e⁻ˣ`), so the residuals are sound for the whole
/// domain.  Each result is confirmed by a 200-bit MPFR `cosh`.  `(|x|_bits,
/// result_bits)`, sorted for binary search.
#[rustfmt::skip]
#[allow(clippy::unreadable_literal)]
const COSH_HARD: [(u64, u64); 10] = [
    (0x3e50000000000000, 0x3ff0000000000001), (0x3e82de32c662873d, 0x3ff000000000002d),
    (0x3ea90b8278768adc, 0x3ff00000000004e7), (0x3ec12d0f92fb5032, 0x3ff00000000024e1),
    (0x3f40cf01d3f9f5e4, 0x3ff000002350f3db), (0x3f610d6a14c0d526, 0x3ff000024591a32b),
    (0x3f7b4ae17e3720ef, 0x3ff0001747116305), (0x3fb2ff1e16810fe7, 0x3ff00b4846f2860f),
    (0x3fe03923f2b47c07, 0x3ff219c1989e3373), (0x3fe52268c6359f0e, 0x3ff39e464805bf01),
];

/// Look `x ≥ 0` up in [`COSH_HARD`], returning its correctly-rounded `cosh`.
#[inline]
fn cosh_database(x: f64) -> Option<f64> {
    let key = x.to_bits();
    COSH_HARD
        .binary_search_by_key(&key, |&(input, _)| input)
        .ok()
        .map(|i| f64::from_bits(COSH_HARD[i].1))
}

/// Correctly-rounded `cosh(x)` for `x ≥ 0` — [`cosh`]'s fallback when the lean
/// leg straddles a rounding boundary.
///
/// `cosh = ½(eˣ + e⁻ˣ) = 2`<sup>`q−1`</sup>`·(m + 2⁻²q/m)` with `m` the two-level
/// `eˣ` mantissa to ≈2⁻¹⁰⁷ ([`exp_two_level_mantissa_accurate`]); the `eˣ + e⁻ˣ`
/// sum never cancels, so the double-double carries the result and
/// [`round_general64`] rounds it soundly.  The handful of sub-2⁻¹⁰⁷ near-ties are
/// caught by [`cosh_database`].  Kept `#[cold]`/out-of-line.
#[cold]
#[inline(never)]
fn cosh_accurate(x: f64) -> f64 {
    if let Some(r) = cosh_database(x) {
        return r;
    }

    let (m, q) = exp_two_level_mantissa_accurate(x);
    let combined = combine(m, q, true);

    // `m + 2⁻²q/m ∈ [1, 2]`; normalize into [1, 2) for the subnormal-safe finish,
    // folding the carry into the `2^(q−1)` exponent.
    let (mantissa, e) = if combined.high >= 2.0 {
        (combined * 0.5, q)
    } else {
        (combined, q - 1)
    };
    round_general64(mantissa, e)
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

/// Upper limit of the result-anchored small-`|x|` series leg for `atanh`
/// (`3/16`, just past the corpus's hardest miss at `|x| ≈ 0.174`).
const ATANH_SMALL: f64 = 0.1875;

/// Upper limit of the result-anchored small-`|x|` series leg for `asinh`
/// (`1/16`, past the corpus's hardest miss at `|x| ≈ 0.060`).
const ASINH_SMALL: f64 = 0.0625;

/// `R(v) = (atanh(x) − x)/x³ = ∑_{k≥0} vᵏ/(2k+3)` (`v = x²`), as a double-double
/// Horner table for the accurate leg.  Degree 22 leaves a truncation `≈2⁻¹¹⁵`
/// relative at `v = ATANH_SMALL² ≈ 0.035`.
#[allow(clippy::unreadable_literal)]
const ATANH_R_DD: [(f64, f64); 23] = [
    (0.3333333333333333, 1.850371707708594e-17),
    (0.2, -1.1102230246251566e-17),
    (0.14285714285714285, 7.93016446160826e-18),
    (0.1111111111111111, 6.1679056923619804e-18),
    (0.09090909090909091, -2.523234146875356e-18),
    (0.07692307692307693, -4.270088556250602e-18),
    (0.06666666666666667, 9.251858538542971e-19),
    (0.058823529411764705, 8.163404592832033e-19),
    (0.05263157894736842, 2.921639538487254e-18),
    (0.047619047619047616, 2.64338815386942e-18),
    (0.043478260869565216, 1.206764157201257e-18),
    (0.04, -8.326672684688674e-19),
    (0.037037037037037035, 2.05596856412066e-18),
    (0.034482758620689655, 4.785444071660157e-19),
    (0.03225806451612903, 8.953411488912552e-19),
    (0.030303030303030304, -8.410780489584519e-19),
    (0.02857142857142857, 8.921435019309293e-19),
    (0.02702702702702703, -1.50030138462859e-18),
    (0.02564102564102564, 8.896017825522087e-19),
    (0.024390243902439025, -8.46206573647223e-19),
    (0.023255813953488372, 3.2273925134452225e-19),
    (0.022222222222222223, -8.480870326997723e-19),
    (0.02127659574468085, 5.167261417803255e-19),
];

/// `S(v) = (asinh(x) − x)/x³` (`v = x²`), the Maclaurin coefficients
/// `(−1)ⁿ (2n)! / (4ⁿ (n!)² (2n+1))` shifted down past the leading `x`, as a
/// double-double Horner table for the accurate leg.  Degree 13 leaves a
/// truncation `≈2⁻¹¹⁷` relative at `v = ASINH_SMALL² ≈ 0.0039`.
#[allow(clippy::unreadable_literal)]
const ASINH_S_DD: [(f64, f64); 14] = [
    (-0.16666666666666666, -9.25185853854297e-18),
    (0.075, 2.7755575615628915e-18),
    (-0.044642857142857144, 9.912705577010326e-19),
    (0.030381944444444444, 3.854941057726238e-19),
    (-0.022372159090909092, 9.462128050782583e-19),
    (0.017352764423076924, -8.006416042969879e-19),
    (-0.01396484375, 6.938893903907229e-19),
    (0.011551800896139705, 8.163404592832033e-19),
    (-0.009761609529194078, -5.478074134663601e-19),
    (0.008390335809616815, 4.130293990420969e-19),
    (-0.0073125258735988454, 3.394024192128536e-19),
    (0.006447210311889649, -3.1225022567582527e-19),
    (-0.005740037670841924, 1.2849803525754126e-19),
    (0.005153309682319905, -3.888173308223878e-19),
];

/// Plain-`f64` `S(v)` for `asinh`'s fast leg.  Degree 8 keeps the truncation
/// `≈2⁻⁷⁶` relative over the branch's range.
#[allow(clippy::unreadable_literal)]
const ASINH_S_FAST: [f64; 9] = [
    -0.16666666666666666,
    0.075,
    -0.044642857142857144,
    0.030381944444444444,
    -0.022372159090909092,
    0.017352764423076924,
    -0.01396484375,
    0.011551800896139705,
    -0.009761609529194078,
];

/// Correction-scaled Ziv gate for `asinh`'s small-`|x|` series fast leg: the
/// absolute error bound is `SCALE · |x|³`, since every error source rides the
/// `x³·S(x²)` correction.  Calibrated to ~26× the measured fast-leg slip
/// (`2⁻⁵³·⁷·x³`).
const ASINH_SMALL_ZIV_SCALE: f64 = 1.776_356_839_400_250_5e-15; // 2⁻⁴⁹

/// Correctly-rounded `atanh(|x|)` for `0 < |x| < ATANH_SMALL`, anchored at the
/// result via the odd series `atanh(x) = x + x³·R(x²)`.
///
/// This is the accurate leg reached from [`atanh_rounded`] on a Ziv straddle.
/// The correction `c = x³·R(x²)` is built as a double-double (its `≈2⁻¹⁰⁴`
/// relative error rides the tiny `c`), then [`round_anchored`] adds the exact
/// `x` and breaks the ½-ulp ties.  Like `log1p`'s small-`|x|` leg, anchoring at
/// the *result* keeps the precision on the tiny `x³·R`, where the double-double
/// argument of the general `ln` path would instead lose `≈−log2|x|` bits of the
/// small part to its leading `1`.
#[inline]
fn atanh_small_accurate(x: f64) -> f64 {
    let v = DoubleDouble::from_product(x, x);
    let (high, low) = ATANH_R_DD[ATANH_R_DD.len() - 1];
    let mut r = DoubleDouble { high, low };
    for &(high, low) in ATANH_R_DD[..ATANH_R_DD.len() - 1].iter().rev() {
        r = r * v + DoubleDouble { high, low };
    }
    let c = (DoubleDouble { high: x, low: 0.0 } * v) * r;
    round_anchored(x, c)
}

/// Correctly-rounded `asinh(|x|)` for `0 < |x| < ASINH_SMALL`, anchored at the
/// result via the odd series `asinh(x) = x + x³·S(x²)`.  Mirrors
/// [`atanh_small_accurate`], but with its own gated series fast leg (which avoids
/// the general path's square root and is a net speedup over `asinh`'s small band).
#[inline]
fn asinh_small(x: f64) -> f64 {
    let v = x * x;
    let x3 = x * v;
    let tail = x3 * crate::poly(v, &ASINH_S_FAST);
    let DoubleDouble { high, low } = fast_sum(x, tail);
    // `|tail| = x³·|S| ≈ x³/6`, so the absolute gate scales with `x³`.
    let err = ASINH_SMALL_ZIV_SCALE * x3;
    let lo = high + (low - err);
    let hi = high + (low + err);
    if lo == hi {
        return lo;
    }
    asinh_small_accurate(x)
}

/// Accurate leg of [`asinh_small`].  Mirrors [`atanh_small_accurate`].
#[inline]
fn asinh_small_accurate(x: f64) -> f64 {
    let v = DoubleDouble::from_product(x, x);
    let (high, low) = ASINH_S_DD[ASINH_S_DD.len() - 1];
    let mut s = DoubleDouble { high, low };
    for &(high, low) in ASINH_S_DD[..ASINH_S_DD.len() - 1].iter().rev() {
        s = s * v + DoubleDouble { high, low };
    }
    let c = (DoubleDouble { high: x, low: 0.0 } * v) * s;
    round_anchored(x, c)
}

/// Inverse hyperbolic sine
///
/// `asinh(x) = ln(x + √(x² + 1))`, odd.  For `|x| < ASINH_SMALL` the
/// result-anchored series [`asinh_small`] applies; for `ASINH_SMALL ≤ |x| ≤ 2²⁷`
/// the log argument is carried as a double-double and fed to [`ln_sqrt_rounded`];
/// for larger `|x|` it collapses to the sqrt-free [`ln_2x_corrected`]
/// (`ln(2|x|) + 1/(4x²)`), which also avoids the `x²` overflow at the top of the
/// range.
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

    let magnitude = if s < ASINH_SMALL {
        asinh_small(s)
    } else if s > LARGE_IHYP {
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
/// log argument fed to [`ln_sqrt_rounded`]; larger `x` collapses to the sqrt-free
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
/// log argument keeps full precision; [`atanh_rounded`] then gives the logarithm
/// (and routes small `|x|` to the result-anchored [`atanh_small_accurate`]).
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

            // (1 + |x|)/(1 − |x|) in double-double, then ½·ln of it.  The lean
            // fast leg is gated; on a straddle the accurate leg is the
            // result-anchored series for small |x|, else the `dint` log.
            let u = ratio_1ps(s);
            atanh_rounded(u, s).copysign(x)
        }
        Some(Ordering::Equal) => f64::INFINITY.copysign(x),
        // |x| > 1 is outside the domain; NaN (the `None` case) propagates.
        _ => f64::NAN,
    }
}
