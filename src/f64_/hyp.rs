use super::double::{
    DoubleDouble, fast_ldexp, fast_sum, round_anchored, round_general_signed64, round_general64,
    sqrt_dd,
};
use super::exp::{exp_two_level_fast, exp_two_level_mantissa_accurate};
use super::{ln_dd_fast, ln_fast_scaled};
use core::cmp::Ordering;

/// `1` as a double-double.
const ONE: DoubleDouble = DoubleDouble {
    high: 1.0,
    low: 0.0,
};

/// Ziv gate for the inverse-hyperbolic fast path, as an absolute bound on the
/// result (`scale · ln(u)`).
///
/// [`super::ln_dd_fast`] differs from the accurate `ln_dd` by <2⁻⁶⁵ *absolute*
/// whatever the result's magnitude — its exact-`z` reduction commits no error, so
/// only the plain-`f64` `ln(1+z)` tail (see `LN_ZIV_EPS` in `log.rs`) plus the
/// `≲2⁻⁶⁷` slip of folding `s.low` into the reduction carry the lean leg's error
/// — and `½·ln` (atanh) only halves it.  `2⁻⁶³` keeps a >4× margin.  Being
/// absolute, the gate forces the accurate fallback only when `|result| ≲ 2⁻¹⁰`,
/// where the lean kernel cannot round correctly anyway.
const IHYP_ZIV_EPS: f64 = 1.084_202_172_485_504_4e-19; // 2^-63

/// Correctly-rounded `atanh(s) = ½·ln(u)` for `u = (1 + s)/(1 − s)` and
/// `ATANH_SMALL ≤ s < 1`, via a two-step Ziv test.  (Smaller `s` takes the cheap
/// [`atanh_small`] series instead, so this leg is never reached there.)
///
/// The lean `½·[super::ln_dd_fast]` is accepted unless it straddles a rounding
/// boundary; on a straddle the sound 128-bit [`super::dint::ln_dd_scaled`] takes
/// over.  The lean leg is `<2⁻⁶⁵` absolute, so its Ziv gate holds the straddle
/// rate to `≈0.5%`.
#[inline]
fn atanh_rounded(u: DoubleDouble) -> f64 {
    let DoubleDouble { high, low } = ln_dd_fast(u) * 0.5;
    let lo = high + (low - IHYP_ZIV_EPS);
    let hi = high + (low + IHYP_ZIV_EPS);
    if lo == hi {
        return lo;
    }

    super::dint::ln_dd_scaled(u.high, u.low, -1)
}

/// Correctly-rounded `ln(x + c)` for the `acosh`/`asinh` argument, where
/// `c = √(x² ∓ 1)` is a double-double and `x > 0`.
///
/// The fast leg gates the lean `super::ln_dd_fast(x + c)`; on a straddle the
/// accurate leg feeds *four* words — `x` and a triple-word `√d` — to the 128-bit
/// `dint` log.  The double-double sqrt (≈2⁻¹⁰⁵, ≈2⁻⁵⁷ ulp through `ln`) is one bit
/// shy of the corpus's hardest ties (≈2⁻⁶²), so a third sqrt word `corr` is
/// refined in and carried alongside `c.high`/`c.low`.
#[inline]
fn ln_sqrt_rounded(x: f64, c: DoubleDouble, d: DoubleDouble) -> f64 {
    let u = c + DoubleDouble { high: x, low: 0.0 };
    let DoubleDouble { high, low } = ln_dd_fast(u);
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

/// `2⁶ = 64`.  For `ASYMP_IHYP ≤ |x| < LARGE_IHYP` `asinh` takes a **sqrt-free**
/// asymptotic fast leg before the double-double sqrt path, so the wide upper
/// stretch of its kernel band pays one division instead of a square root.  Below
/// 64 the asymptotic would need too many terms (and the `f64` correction loses its
/// soundness margin); above `LARGE_IHYP` a single correction term suffices
/// ([`ln_2x_corrected`]).
const ASYMP_IHYP: f64 = 64.0;

/// Asymptotic correction `f(v) = a₁v + a₂v² + a₃v³ + a₄v⁴ + a₅v⁵` of
/// `ln((1 + √(1 + v))/2)` with `v = 1/x²`, for `asinh(x) = ln(2x) + f(v)` —
/// low-degree-first for `crate::poly`, evaluated `v·poly(v)`.  The dropped `a₆v⁶`
/// is `<2⁻⁷²` for `|x| ≥ 64`, far inside the [`IHYP_ZIV_EPS`] gate.
const ASINH_ASYMP: [f64; 5] = [
    0.25,             // 1/4
    -0.093_75,        // −3/32
    5.0 / 96.0,       // 5/96
    -0.034_179_687_5, // −35/1024
    63.0 / 2560.0,    // 63/2560
];

/// Sqrt-free asymptotic fast leg for `ASYMP_IHYP ≤ |x| < LARGE_IHYP`:
/// `asinh(x) = ln(2x) + f(1/x²)`.  `ln(2x)` comes from the scaled reduction; the
/// asymptotic correction `f` (up to `≈6e-5` at `x = 64`, far above `ulp(high)`) is
/// added in with one 2Sum.  Returns the raw pair the Ziv test consumes.
#[inline]
fn asinh_asymptotic_pair(s: f64) -> DoubleDouble {
    let v = 1.0 / (s * s);
    let correction = v * crate::poly(v, &ASINH_ASYMP);

    let DoubleDouble { high, low } = ln_fast_scaled(s, 1);
    let DoubleDouble { high, low: cl } = DoubleDouble::from_sum(high, correction);
    DoubleDouble {
        high,
        low: cl + low,
    }
}

/// `asinh(|x|)` magnitude for the kernel band `[ASINH_SMALL, LARGE_IHYP]`: the
/// sqrt-free [`asinh_asymptotic_pair`], Ziv-tested, for `|x| ≥ ASYMP_IHYP`, else
/// (and on a straddle) the double-double sqrt path [`ln_sqrt_rounded`].
///
/// `acosh` deliberately does *not* share this asymptotic: its kernel band reaches
/// only `2¹¹` and is dominated by the near-1 region, so the asymptotic's large-`x`
/// payoff is too thin a slice to cover the threshold branch's misprediction on a
/// log-uniform argument — see [`acosh`].
#[inline]
fn asinh_mid(s: f64) -> f64 {
    if s >= ASYMP_IHYP {
        let DoubleDouble { high, low } = asinh_asymptotic_pair(s);
        let lo = high + (low - IHYP_ZIV_EPS);
        let hi = high + (low + IHYP_ZIV_EPS);
        if lo == hi {
            return lo;
        }
    }
    let d = DoubleDouble::from_product(s, s) + ONE;
    ln_sqrt_rounded(s, sqrt_dd(d), d)
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

/// Reduction-exponent cutoff above which `cosh`/`sinh` drop the `e⁻ˣ` term and
/// return `½eˣ` directly.
///
/// `eˣ = m·2^q`, so the `e⁻ˣ` correction to the `[1, 2)` mantissa is
/// `c = 2⁻²q/m ≈ m·e⁻²ˣ`.  At `q = 35` (`|x| ≳ 35·ln2 ≈ 24.3`) that is `≤ 2⁻⁷⁰` —
/// far below both the fast leg's own `≈2⁻⁶⁴·⁷` slip and the [`HYP_ZIV_EPS`] gate —
/// so `½(eˣ ± e⁻ˣ)` rounds identically to `½eˣ` and the gate margin is unchanged.
/// Dropping the term there skips [`combine_fast`]'s division and double-double add
/// on the ~96% of a value-uniform `[−710, 710]` sweep that lands above the cutoff;
/// the rare Ziv straddle still defers to the accurate path, which forms the full
/// `e⁻ˣ`.  (`q ≥ 33` already suffices for soundness; `35` keeps the leg's full
/// margin so the fallback rate is byte-for-byte the combine path's.)
const HYP_HALF_EXP_Q: i64 = 35;

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
    // path: lean `eˣ` mantissa accepted by a Ziv test, else the accurate one.  For
    // `q ≥ HYP_HALF_EXP_Q` the `e⁻ˣ` term is below the gate, so `m` alone (= ½eˣ)
    // is gated and the `combine_fast` division is skipped.
    let (m, q) = exp_two_level_fast(x);
    let mantissa = if q >= HYP_HALF_EXP_Q {
        m
    } else {
        combine_fast(m, q, true)
    };
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

    // |x| < ¼: `½(eˣ − e⁻ˣ)` cancels catastrophically (the linear terms of eˣ and
    // e⁻ˣ cancel, leaving ≈2x), so the `m − 2⁻²q/m` mantissa loses the result's
    // leading bits.  Go *result-anchored* via the odd series — see [`sinh_small`].
    if s < SINH_SMALL {
        return sinh_small(s).copysign(x);
    }

    // |x| ≥ ¼: sinh(x) = ½(eˣ − e⁻ˣ) = 2^(q−1)·(m − 2⁻²q/m); the residual
    // cancellation (`t < 0.6·m`) is mild enough for the double-double.  Fast path:
    // lean mantissa accepted by a Ziv test, as in `cosh`.  For `q ≥ HYP_HALF_EXP_Q`
    // the `e⁻ˣ` term is below the gate, so `m` alone (= ½eˣ) is gated and the
    // `combine_fast` division is skipped.
    let (m, q) = exp_two_level_fast(s);
    let mantissa = if q >= HYP_HALF_EXP_Q {
        m
    } else {
        combine_fast(m, q, false)
    };
    let lo = mantissa.high + (mantissa.low - HYP_ZIV_EPS);
    let hi = mantissa.high + (mantissa.low + HYP_ZIV_EPS);
    if lo == hi {
        return fast_ldexp(lo, q - 1).copysign(x);
    }

    sinh_accurate(s).copysign(x)
}

/// Upper limit of `sinh`'s result-anchored small-`|x|` series leg.
const SINH_SMALL: f64 = 0.25;

/// `S(v) = (sinh(x) − x)/x³ = ∑ vᵏ/(2k+3)!` (`v = x²`), low-degree first, as a
/// double-double Horner table for [`sinh_small_accurate`].  Degree 11 leaves a
/// truncation ≈2⁻¹³⁶ relative at `v = SINH_SMALL² = 0.0625`.
#[allow(clippy::unreadable_literal)]
const SINH_S_DD: [(f64, f64); 12] = [
    (0.16666666666666666, 9.25185853854297e-18),
    (0.008333333333333333, 1.1564823173178714e-19),
    (0.0001984126984126984, 1.7209558293420705e-22),
    (2.7557319223985893e-6, -1.858393274046472e-22),
    (2.505210838544172e-8, -1.448814070935912e-24),
    (1.6059043836821613e-10, 1.2585294588752098e-26),
    (7.647163731819816e-13, 7.03872877733453e-30),
    (2.8114572543455206e-15, 1.6508842730861433e-31),
    (8.22063524662433e-18, 2.2141894119604265e-34),
    (1.9572941063391263e-20, -1.3643503830087908e-36),
    (3.868170170630684e-23, -8.843177655482344e-40),
    (6.446950284384474e-26, -1.9330404233703465e-42),
];

/// Plain-`f64` `S(v)` for [`sinh_small`]'s fast leg; degree 7 (its `v⁸/19!`
/// truncation is ≈2⁻⁹⁰ relative over `v < 0.0625`, far under the `f64` floor).
#[allow(clippy::unreadable_literal)]
const SINH_S_FAST: [f64; 8] = [
    0.16666666666666666,
    0.008333333333333333,
    0.0001984126984126984,
    2.7557319223985893e-6,
    2.505210838544172e-8,
    1.6059043836821613e-10,
    7.647163731819816e-13,
    2.8114572543455206e-15,
];

/// Correction-scaled Ziv gate for [`sinh_small`]'s series fast leg: the absolute
/// error bound is `SCALE · |x|³`, since every error source rides the `x³·S(x²)`
/// correction (the same shape as `asinh`).  ≈26× the measured fast-leg slip.
const SINH_SMALL_ZIV_SCALE: f64 = 1.776_356_839_400_250_5e-15; // 2⁻⁴⁹

/// Correctly-rounded `sinh(|x|)` for `0 < |x| < SINH_SMALL` via the
/// result-anchored odd series `sinh(x) = x + x³·S(x²)` — `½(eˣ − e⁻ˣ)` cancels
/// here, so this anchors at the exact `x` instead (the `asinh`/`expm1` template).
/// Lean fast leg (plain-`f64` [`SINH_S_FAST`], `x³`-scaled gate), falling to the
/// double-double [`sinh_small_accurate`] on a straddle.
#[inline]
fn sinh_small(x: f64) -> f64 {
    let v = x * x;
    let x3 = x * v;
    let tail = x3 * crate::poly(v, &SINH_S_FAST);
    let DoubleDouble { high, low } = fast_sum(x, tail);
    let err = SINH_SMALL_ZIV_SCALE * x3;
    let lo = high + (low - err);
    let hi = high + (low + err);
    if lo == hi {
        return lo;
    }
    sinh_small_accurate(x)
}

/// Accurate leg of [`sinh_small`]: the correction `c = x³·S(x²)` as a
/// double-double ([`SINH_S_DD`] Horner), then [`round_anchored`] adds the exact
/// `x` and breaks the ½-ulp ties.  Mirrors `asinh_small_accurate`; the
/// double-double resolves the whole branch (no database entries needed here).
#[inline]
fn sinh_small_accurate(x: f64) -> f64 {
    let v = DoubleDouble::from_product(x, x);
    let (high, low) = SINH_S_DD[SINH_S_DD.len() - 1];
    let mut s = DoubleDouble { high, low };
    for &(high, low) in SINH_S_DD[..SINH_S_DD.len() - 1].iter().rev() {
        s = s * v + DoubleDouble { high, low };
    }
    let c = (DoubleDouble { high: x, low: 0.0 } * v) * s;
    round_anchored(x, c)
}

/// Hard-to-round database for [`sinh_accurate`]: non-negative inputs (`|x| ≥ ¼`)
/// whose `sinh` lies within the double-double path's reach of an `f64` midpoint,
/// mapped (by bit pattern of `|x|`) to their correctly-rounded results.  Residuals
/// metallic's accurate path leaves on the `sinh.wc` corpus; the path is
/// db-complete (its error stays far below the corpus's ≥40-bit ≈2⁻⁹⁴ threshold),
/// so this is sound for the whole `|x| ≥ ¼` domain.  Each confirmed by a 200-bit
/// MPFR `sinh`.  `(|x|_bits, result_bits)`, sorted for binary search.
#[rustfmt::skip]
#[allow(clippy::unreadable_literal)]
const SINH_HARD: [(u64, u64); 2] = [
    (0x3fd4169f234f23b9, 0x3fd46b7b3b358f99), (0x3fd6660974af4f3a, 0x3fd6dbcf9dad0171),
];

/// Look `x ≥ ¼` up in [`SINH_HARD`], returning its correctly-rounded `sinh`.
#[inline]
fn sinh_database(x: f64) -> Option<f64> {
    let key = x.to_bits();
    SINH_HARD
        .binary_search_by_key(&key, |&(input, _)| input)
        .ok()
        .map(|i| f64::from_bits(SINH_HARD[i].1))
}

/// Correctly-rounded `sinh(x)` for `x ≥ ¼` — [`sinh`]'s fallback when the lean
/// leg straddles a rounding boundary.
///
/// `sinh = ½(eˣ − e⁻ˣ) = 2`<sup>`q−1`</sup>`·(m − 2⁻²q/m)` with `m` the two-level
/// `eˣ` mantissa to ≈2⁻¹⁰⁷ ([`exp_two_level_mantissa_accurate`]).  For `x ≥ ¼` the
/// `e⁻ˣ` term is at most `0.6·eˣ`, so the subtraction loses ≤ 1 bit and the
/// double-double carries the result; [`round_general_signed64`] normalizes the
/// `(0, 2)` mantissa and rounds it soundly.  The sub-2⁻¹⁰⁷ near-ties go in
/// [`sinh_database`].  Kept `#[cold]`/out-of-line.
#[cold]
#[inline(never)]
fn sinh_accurate(x: f64) -> f64 {
    if let Some(r) = sinh_database(x) {
        return r;
    }

    let (m, q) = exp_two_level_mantissa_accurate(x);
    let combined = combine(m, q, false);
    round_general_signed64(combined, q - 1)
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

    // |x| < ⅛: forming expm1(2x) = 2^q·m − 1 would cancel (2x is small), so go
    // *result-anchored* via the odd series — see [`tanh_small`].
    if s < TANH_SMALL {
        return tanh_small(s).copysign(x);
    }

    // |x| ∈ [⅛, 20): tanh = E/(E + 2), E = e²ˣ − 1 = m·2^q − 1.  Fast path: the
    // lean `e²ˣ` mantissa ([`exp_two_level_fast`]), the same `E/(E + 2)` as the
    // accurate leg, accepted by a Ziv test; else [`tanh_accurate`].
    let (m, q) = exp_two_level_fast(2.0 * s);
    let result = tanh_combine(m, q);
    let lo = result.high + (result.low - TANH_ZIV_EPS);
    let hi = result.high + (result.low + TANH_ZIV_EPS);
    if lo == hi {
        return lo.copysign(x);
    }

    tanh_accurate(s).copysign(x)
}

/// Form `tanh = E/(E + 2)` from `e²ˣ = 2`<sup>`q`</sup>`·m`, with `E = e²ˣ − 1`.
/// Shared by [`tanh`]'s fast leg and [`tanh_accurate`] — only the source of
/// `(m, q)` (lean vs ≈2⁻¹⁰⁷ mantissa) differs.
///
/// `E/(E + 2)` never cancels for `2x ≥ ¼` (`E ≥ 0.28`, `E + 2` the larger), so
/// the double-double carries the result.  The `− 1` is exact only in the low
/// word once `q` is large (`tanh → 1`), but there the result error rides
/// `(1 − tanh²)/2 → 0`, so the leg stays accurate.
#[inline]
fn tanh_combine(m: DoubleDouble, q: i64) -> DoubleDouble {
    let e = m * crate::exp2i(q)
        + DoubleDouble {
            high: -1.0,
            low: 0.0,
        };
    e * (e + DoubleDouble {
        high: 2.0,
        low: 0.0,
    })
    .recip()
}

/// Ziv gate for [`tanh`]'s fast leg, as an absolute bound on the result.
///
/// `tanh = 1 − 2/(e²ˣ + 1)`, so a relative error `δ` in `e²ˣ` propagates as
/// `δ·(1 − tanh²)/2 ≤ δ/2`.  The fast `e²ˣ` mantissa slips ≈2⁻⁶⁴·⁷ relative
/// (the [`HYP_ZIV_EPS`] leg), so the result is good to ≈2⁻⁶⁵·⁷ absolute; the
/// `E/(E + 2)` double-double adds only ≈2⁻¹⁰⁶.  `2⁻⁶²` keeps a >10× margin.
const TANH_ZIV_EPS: f64 = 2.168_404_344_971_009e-19; // 2^-62

/// Upper limit of `tanh`'s result-anchored small-`|x|` series leg.
const TANH_SMALL: f64 = 0.125;

/// `T(v) = (tanh(x) − x)/x³ = ∑ Tₖ vᵏ` (`v = x²`), low-degree first, as a
/// double-double Horner table for [`tanh_small_accurate`].  `Tₖ = aₖ₊₁` of the
/// `tanh` series `aₖ = −(∑ aᵢaⱼ)/(2k+1)` (`a₀ = 1`); unlike `sinh`'s factorially
/// decaying coefficients these decay only geometrically (≈0.41ᵏ), so degree 15 is
/// needed for the `v¹⁶/…` truncation (≈2⁻¹¹⁷ relative at `v = TANH_SMALL² =
/// 0.0156`) to clear the double-double floor.
#[allow(clippy::unreadable_literal)]
const TANH_T_DD: [(f64, f64); 16] = [
    (-0.3333333333333333, -1.850371707708594e-17),
    (0.13333333333333333, 1.8503717077085942e-18),
    (-0.053968253968253964, -4.383618688500122e-18),
    (0.0218694885361552, 3.2956686566529393e-18),
    (-0.008863235529902196, -9.71422895804976e-19),
    (0.0035921280365724807, 3.0829850815353886e-19),
    (-0.0014558343870513181, -1.5469550809574026e-19),
    (0.000590027440945586, 3.478690842383652e-20),
    (-0.00023912911424355248, -3.564613898329782e-21),
    (9.69153795692945e-5, 7.313864280505558e-21),
    (-3.927832388331683e-5, -1.3737015743076767e-21),
    (1.5918905069328964e-5, 1.0427554807190543e-21),
    (-6.451689215655431e-6, -1.1519922496640055e-22),
    (2.6147711512907542e-6, 3.3037961741415215e-22),
    (-1.0597268320104654e-6, -2.367052550521363e-24),
    (4.294911078273806e-7, 1.1643520863702653e-23),
];

/// Plain-`f64` `T(v)` for [`tanh_small`]'s fast leg; degree 8 (its `v⁹/…`
/// truncation is ≈2⁻⁶⁵ relative over `v < 0.0156`, far under the `f64` floor).
#[allow(clippy::unreadable_literal)]
const TANH_T_FAST: [f64; 9] = [
    -0.3333333333333333,
    0.13333333333333333,
    -0.053968253968253964,
    0.0218694885361552,
    -0.008863235529902196,
    0.0035921280365724807,
    -0.0014558343870513181,
    0.000590027440945586,
    -0.00023912911424355248,
];

/// Correction-scaled Ziv gate for [`tanh_small`]'s series fast leg: the absolute
/// error bound is `SCALE · |x|³`, since every error source rides the `x³·T(x²)`
/// correction (the `asinh`/`sinh` shape).  ≈25× the measured fast-leg slip.
const TANH_SMALL_ZIV_SCALE: f64 = 1.776_356_839_400_250_5e-15; // 2⁻⁴⁹

/// Correctly-rounded `tanh(|x|)` for `0 < |x| < TANH_SMALL` via the
/// result-anchored odd series `tanh(x) = x + x³·T(x²)`.  Lean fast leg
/// (plain-`f64` [`TANH_T_FAST`], `x³`-scaled gate), falling to the double-double
/// [`tanh_small_accurate`] on a straddle.  Mirrors `sinh_small`.
#[inline]
fn tanh_small(x: f64) -> f64 {
    let v = x * x;
    let x3 = x * v;
    let tail = x3 * crate::poly(v, &TANH_T_FAST);
    let DoubleDouble { high, low } = fast_sum(x, tail);
    let err = TANH_SMALL_ZIV_SCALE * x3;
    let lo = high + (low - err);
    let hi = high + (low + err);
    if lo == hi {
        return lo;
    }
    tanh_small_accurate(x)
}

/// Accurate leg of [`tanh_small`]: `c = x³·T(x²)` as a double-double
/// ([`TANH_T_DD`] Horner), then [`round_anchored`] adds the exact `x`.
#[inline]
fn tanh_small_accurate(x: f64) -> f64 {
    let v = DoubleDouble::from_product(x, x);
    let (high, low) = TANH_T_DD[TANH_T_DD.len() - 1];
    let mut t = DoubleDouble { high, low };
    for &(high, low) in TANH_T_DD[..TANH_T_DD.len() - 1].iter().rev() {
        t = t * v + DoubleDouble { high, low };
    }
    let c = (DoubleDouble { high: x, low: 0.0 } * v) * t;
    round_anchored(x, c)
}

/// Hard-to-round database for [`tanh_accurate`]: non-negative inputs (`⅛ ≤ |x| <
/// 20`) whose `tanh` lies within the double-double path's reach of an `f64`
/// midpoint, mapped (by bit pattern of `|x|`) to their correctly-rounded results.
/// Residuals metallic's accurate path leaves on the `tanh.wc` corpus; the path is
/// db-complete (error far below the corpus's ≥40-bit ≈2⁻⁹⁴ threshold), so this is
/// sound for the whole `⅛ ≤ |x| < 20` domain.  Each confirmed by a 200-bit MPFR
/// `tanh`.  `(|x|_bits, result_bits)`, sorted for binary search.
#[rustfmt::skip]
#[allow(clippy::unreadable_literal)]
const TANH_HARD: [(u64, u64); 2] = [
    (0x3fcac343b179fec4, 0x3fca612499c53078), (0x3fd291c601a05276, 0x3fd210b7d0c03743),
];

/// Look `x` (`⅛ ≤ x < 20`) up in [`TANH_HARD`], returning its correctly-rounded
/// `tanh`.
#[inline]
fn tanh_database(x: f64) -> Option<f64> {
    let key = x.to_bits();
    TANH_HARD
        .binary_search_by_key(&key, |&(input, _)| input)
        .ok()
        .map(|i| f64::from_bits(TANH_HARD[i].1))
}

/// Correctly-rounded `tanh(x)` for `⅛ ≤ x < 20` via `tanh = E/(E + 2)`,
/// `E = expm1(2x) = 2^q·m − 1` with `m` the two-level `eˣ` mantissa to ≈2⁻¹⁰⁷
/// ([`exp_two_level_mantissa_accurate`]).  For `2x ≥ ¼` the `− 1` cancels ≤ ~2
/// bits and the quotient never cancels, so the double-double carries the result;
/// [`round_general_signed64`] rounds it soundly and the sub-2⁻¹⁰⁷ near-ties go in
/// [`tanh_database`].  Kept `#[cold]`/out-of-line.
#[cold]
#[inline(never)]
fn tanh_accurate(x: f64) -> f64 {
    if let Some(r) = tanh_database(x) {
        return r;
    }

    let (m, q) = exp_two_level_mantissa_accurate(2.0 * x);
    round_general_signed64(tanh_combine(m, q), 0)
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

/// `1/3` as a double-double — the exact leading coefficient of `atanh`'s series
/// (`ATANH_R_DD[0]`), carried in full precision by [`atanh_small_eval`] so the
/// dominant `x³/3` term commits no `f64` rounding.
const FRAC_1_3_DD: DoubleDouble = DoubleDouble {
    high: 0.333_333_333_333_333_3,
    low: 1.850_371_707_708_594e-17,
};

/// Plain-`f64` `R2(v) = (R(v) − 1/3)/v = ∑_{k≥0} vᵏ/(2k+5)` for `atanh`'s
/// small-`|x|` fast leg (`v = x²`, `R = (atanh(x) − x)/x³`).  The exact Taylor
/// coefficients `1/(2k+5)` (the high words of [`ATANH_R_DD`] shifted down one);
/// degree 10 leaves a truncation `≈2⁻⁶³` relative on `R` at `v = ATANH_SMALL² ≈
/// 0.035`.  Only this small `v·R2` correction to the double-double `1/3` rides
/// the `f64` rounding, so the leg's slip stays near `2⁻⁵⁹·|x|³`.
#[allow(clippy::unreadable_literal)]
const ATANH_R2_FAST: [f64; 11] = [
    0.2,
    0.14285714285714285,
    0.1111111111111111,
    0.09090909090909091,
    0.07692307692307693,
    0.06666666666666667,
    0.058823529411764705,
    0.05263157894736842,
    0.047619047619047616,
    0.043478260869565216,
    0.04,
];

/// Ziv gate for `atanh`'s small-`|x|` series fast leg, `err = |x|·(x⁴·REL +
/// FLOOR)` (CORE-MATH's `atanh` shape).  Because [`atanh_small_eval`] carries the
/// exact `x` and `x³/3`, its polynomial slip rides the `x⁵` remainder
/// (`REL·x⁵`); the `FLOOR·|x|` term covers the `≈2⁻¹⁰⁶·|x|` loss when the anchor
/// fold `s.low + tail.low` drops `tail.low` near a half-ulp tie (where the
/// dropped bits decide the rounding) — without it the tightest near-ties at tiny
/// `|x|` mis-round.  Both calibrated to ≥2× the MPFR-measured slip by
/// `ziv_soundness::atanh_small_leg_is_sound`.
const ATANH_SMALL_ZIV_REL: f64 = 8.881_784_197_001_252e-16; // 2⁻⁵⁰
/// Absolute-error floor of the [`ATANH_SMALL_ZIV_REL`] gate; see there.
const ATANH_SMALL_ZIV_FLOOR: f64 = 1.972_152_263_052_530_6e-31; // 2⁻¹⁰²

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

/// The un-rounded double-double of `atanh`'s small-`|x|` fast leg, `x + x³·R(x²)`
/// with `R(v) = 1/3 + v·R2(v)`.
///
/// `x²` and `x³` are carried as double-doubles and the leading `1/3` enters as
/// the exact [`FRAC_1_3_DD`], so only the small `v·R2(v)` correction (R2 from
/// [`ATANH_R2_FAST`], plain `f64`) rounds — leaving the dominant `x³/3` term and
/// the anchor `x` exact.  The slip therefore stays near `2⁻⁵⁹·|x|³`, well inside
/// the [`ATANH_SMALL_ZIV_REL`] gate's `x⁵` term.  Shared with `ziv_soundness`.
#[inline]
fn atanh_small_eval(x: f64) -> DoubleDouble {
    let v = DoubleDouble::from_product(x, x); // x² exact
    let x3 = v * x; // x³ to double-double
    // R(v) = 1/3 + v·R2(v); the f64 correction is ≲ 0.012, far under 1/3.
    let r = FRAC_1_3_DD.add_ordered(DoubleDouble {
        high: v.high * crate::poly(v.high, &ATANH_R2_FAST),
        low: 0.0,
    });
    let tail = x3 * r;
    let s = fast_sum(x, tail.high);
    DoubleDouble {
        high: s.high,
        low: s.low + tail.low,
    }
}

/// Correctly-rounded `atanh(|x|)` for `0 < |x| < ATANH_SMALL` via the cheap
/// result-anchored series fast leg `x + x³·R(x²)` — no `(1 + x)/(1 − x)` division
/// and no `ln`, the analogue of CORE-MATH's direct `|x| < ¼` branch and a mirror
/// of [`asinh_small`].  The gate is the `x⁵`-shaped [`ATANH_SMALL_ZIV_REL`]; on a
/// straddle the result-anchored [`atanh_small_accurate`] takes over.
#[inline]
fn atanh_small(x: f64) -> f64 {
    let DoubleDouble { high, low } = atanh_small_eval(x);
    let x2 = x * x;
    let err = x * crate::fast_mul_add(x2 * x2, ATANH_SMALL_ZIV_REL, ATANH_SMALL_ZIV_FLOOR);
    let lo = high + (low - err);
    let hi = high + (low + err);
    if lo == hi {
        return lo;
    }
    atanh_small_accurate(x)
}

/// Correctly-rounded `atanh(|x|)` for `0 < |x| < ATANH_SMALL`, anchored at the
/// result via the odd series `atanh(x) = x + x³·R(x²)`.
///
/// This is the accurate leg reached from [`atanh_small`] on a Ziv straddle.
/// The correction `c = x³·R(x²)` is built as a double-double (its `≈2⁻¹⁰⁴`
/// relative error rides the tiny `c`), then [`round_anchored`] adds the exact
/// `x` and breaks the ½-ulp ties.  Like `log1p`'s small-`|x|` leg, anchoring at
/// the *result* keeps the precision on the tiny `x³·R`, where the double-double
/// argument of the general `ln` path would instead lose `≈−log2|x|` bits of the
/// small part to its leading `1`.
///
/// `#[cold]`/never-inlined so the degree-22 double-double Horner stays out of
/// [`atanh_small`]'s hot path — keeping it lean matters in the value-uniform
/// bench, where mixing it in line pollutes the cache for the `(1 + x)/(1 − x)`
/// leg too.
#[cold]
#[inline(never)]
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
        asinh_mid(s)
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

    // The sqrt-free asymptotic ([`asinh_mid`]) is *not* used here: acosh's kernel
    // band `[1, 2¹¹)` is dominated by the near-1 region, so its only payoff would be
    // the large-`x` tail — too thin a slice to cover the threshold branch's
    // misprediction on a log-uniform argument.  (`asinh`'s band reaches `2²⁶`, so
    // there the asymptotic broadly pays.)
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
/// `atanh(x) = ½·ln((1 + x)/(1 − x))` for `|x| < 1`, odd.  For `|x| < ATANH_SMALL`
/// the cheap result-anchored series [`atanh_small`] applies (no division, no
/// `ln`); otherwise both `1 ± x` are formed exactly as double-doubles (2Sum) and
/// divided in double-double — so the log argument keeps full precision — and
/// [`atanh_rounded`] gives the logarithm.
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

            let magnitude = if s < ATANH_SMALL {
                // Cheap direct series, gated; straddle → result-anchored series.
                atanh_small(s)
            } else {
                // (1 + |x|)/(1 − |x|) in double-double, then ½·ln of it.  The
                // lean fast leg is gated; on a straddle the `dint` log takes over.
                atanh_rounded(ratio_1ps(s))
            };
            magnitude.copysign(x)
        }
        Some(Ordering::Equal) => f64::INFINITY.copysign(x),
        // |x| > 1 is outside the domain; NaN (the `None` case) propagates.
        _ => f64::NAN,
    }
}

/// MPFR-certified soundness of the small-`|x|` fast-leg Ziv gates.  A gate must
/// exceed the leg's true error with margin (here ≥2×), or a confident `lo == hi`
/// could certify a value on the wrong side of a rounding boundary.  Run with
/// `--features mpfr`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::*;
    use rug::Float;

    fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Worst-case `|leg(x) − f(x)| / gate(x)` over `[lo, hi]` vs a 250-bit MPFR
    /// reference `f`, with `leg` the (folded) double-double the Ziv test actually
    /// sees and `gate` its half-width.  A ratio `< 0.5` everywhere certifies the
    /// 2× soundness margin.  Returns `(worst_ratio, worst_x)`.
    fn worst_ratio(
        lo: f64,
        hi: f64,
        n: u64,
        leg: impl Fn(f64) -> DoubleDouble,
        gate: impl Fn(f64) -> f64,
        f: impl Fn(&Float) -> Float,
    ) -> (f64, f64) {
        let (lb, hb) = (lo.to_bits(), hi.to_bits());
        let mut worst = 0.0_f64;
        let mut worst_x = lo;
        for i in 0..n {
            let b = lb + mix(i) % (hb - lb);
            let x = f64::from_bits(b);
            let e = leg(x);
            let got = Float::with_val(250, e.high) + Float::with_val(250, e.low);
            let truth = f(&Float::with_val(250, x));
            let abs = Float::with_val(250, &got - &truth).abs().to_f64();
            let ratio = abs / gate(x);
            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        (worst, worst_x)
    }

    /// The [`atanh_small`] gate must cover [`atanh_small_eval`]'s true error
    /// (folded `high + low`, exactly as the Ziv test reads it) over its operating
    /// range `[2⁻²⁷, ATANH_SMALL)` — including the tiny-`|x|` near-ties where the
    /// `FLOOR·|x|` term carries the anchor fold's dropped `tail.low`.
    #[test]
    fn atanh_small_leg_is_sound() {
        let gate = |x: f64| {
            let x2 = x * x;
            x * crate::fast_mul_add(x2 * x2, ATANH_SMALL_ZIV_REL, ATANH_SMALL_ZIV_FLOOR)
        };
        let (worst, x) = worst_ratio(
            7.450_580_596_923_828e-9,
            ATANH_SMALL,
            8_000_000,
            atanh_small_eval,
            gate,
            |x| x.clone().atanh(),
        );
        println!("atanh_small_eval: worst |err|/gate = {worst:.4} at x={x:e}");
        assert!(
            worst < 0.5,
            "atanh_small gate covers only {:.2}× the slip at x={x:e}",
            1.0 / worst
        );
    }

    /// Same check for `asinh`'s small-`|x|` fast leg ([`ASINH_S_FAST`]), gate
    /// `ASINH_SMALL_ZIV_SCALE · |x|³`.
    #[test]
    fn asinh_small_leg_is_sound() {
        let leg = |x: f64| {
            let v = x * x;
            let tail = (x * v) * crate::poly(v, &ASINH_S_FAST);
            fast_sum(x, tail)
        };
        let (worst, x) = worst_ratio(
            7.450_580_596_923_828e-9,
            ASINH_SMALL,
            8_000_000,
            leg,
            |x| ASINH_SMALL_ZIV_SCALE * (x * x * x),
            |x| x.clone().asinh(),
        );
        println!("asinh_small leg: worst |err|/gate = {worst:.4} at x={x:e}");
        assert!(
            worst < 0.5,
            "asinh_small gate covers only {:.2}× the slip at x={x:e}",
            1.0 / worst
        );
    }

    /// The inverse-hyperbolic main leg's gate ([`IHYP_ZIV_EPS`]) must cover the
    /// error of [`super::ln_dd_fast`] — in particular the `≲2⁻⁶⁷` slip from
    /// folding the double-double's low word into the exact-`z` reduction (which
    /// replaced the old `s.low/s.high` division).  Checked through `atanh`'s
    /// consumer (`u = (1+x)/(1−x)` via [`ratio_1ps`], then `½·ln u`), where the
    /// log argument reaches the widest magnitudes over `[ATANH_SMALL, 1)`.
    #[test]
    fn atanh_main_leg_is_sound() {
        let leg = |x: f64| ln_dd_fast(ratio_1ps(x)) * 0.5;
        let (worst, x) = worst_ratio(
            ATANH_SMALL,
            0.999_999_999,
            8_000_000,
            leg,
            |_| IHYP_ZIV_EPS,
            |x| x.clone().atanh(),
        );
        println!("atanh main leg: worst |err|/gate = {worst:.4} at x={x:e}");
        assert!(
            worst < 0.5,
            "atanh main-leg gate covers only {:.2}× the slip at x={x:e}",
            1.0 / worst
        );
    }

    /// Same check for the `asinh`/`acosh` middle band ([`ln_sqrt_rounded`]):
    /// `ln(x + √(x² + 1))` via [`super::ln_dd_fast`], gate [`IHYP_ZIV_EPS`], over
    /// `[ASINH_SMALL, LARGE_IHYP]`.
    #[test]
    fn asinh_main_leg_is_sound() {
        let leg = |x: f64| {
            let d = DoubleDouble::from_product(x, x) + ONE;
            ln_dd_fast(sqrt_dd(d) + DoubleDouble { high: x, low: 0.0 })
        };
        let (worst, x) = worst_ratio(
            ASINH_SMALL,
            LARGE_IHYP,
            8_000_000,
            leg,
            |_| IHYP_ZIV_EPS,
            |x| x.clone().asinh(),
        );
        println!("asinh main leg: worst |err|/gate = {worst:.4} at x={x:e}");
        assert!(
            worst < 0.5,
            "asinh main-leg gate covers only {:.2}× the slip at x={x:e}",
            1.0 / worst
        );
    }

    /// The sqrt-free [`asinh_asymptotic_pair`] must clear the [`IHYP_ZIV_EPS`] gate
    /// over its operating range `[ASYMP_IHYP, LARGE_IHYP]`.  The truncated `a₆v⁶`
    /// term and the `f64` correction's rounding are both largest at the bottom of
    /// the range (`|x| = 64`), so this is where the gate is tightest.
    #[test]
    fn asinh_asymptotic_leg_is_sound() {
        let (worst, x) = worst_ratio(
            ASYMP_IHYP,
            LARGE_IHYP,
            8_000_000,
            asinh_asymptotic_pair,
            |_| IHYP_ZIV_EPS,
            |x| x.clone().asinh(),
        );
        println!("asinh asymptotic leg: worst |err|/gate = {worst:.4} at x={x:e}");
        assert!(
            worst < 0.5,
            "asinh asymptotic-leg gate covers only {:.2}× the slip at x={x:e}",
            1.0 / worst
        );
    }
}
