//! Power function `xʸ = 2^(y·log₂x)`, correctly rounded.
//!
//! The hot path is the lean `2^(y·log₂x)` of [`pow_fast`] under a Ziv gate; the
//! rare straddles and the over/underflow / subnormal edges defer to
//! [`pow_accurate`](super::pow_accurate::pow_accurate), a faithful port of
//! CORE-MATH's two final Ziv iterations (128-bit `Dint`, then 256-bit `Qint`)
//! plus its exact/midpoint detector — which together make every result
//! bit-identical to the `core-math` oracle, including the rational-exponent
//! ties-to-even cases.
//!
//! The double-double `log₂`/`exp2` kernels here are also shared with the f32
//! [`powf`](crate::f32_::powf): they already run in f64 double-double, since even
//! for an f32 result the error in `log₂x` is amplified by `y`.
#![allow(clippy::unreadable_literal, clippy::excessive_precision)]

use super::EXP_SHIFT;
use super::double::{DoubleDouble, fast_ldexp, fast_sum};
use core::cmp::Ordering;
use core::f64::consts::FRAC_1_SQRT_2;
use core::num::FpCategory;

/// Double-double minimax of `2·log₂e·atanh(t)/t` in `u = t²`, low-degree first
///
/// Degree 12, max error `2⁻⁹⁴` on `u ∈ [0, 0.0295]` (i.e. `|t| ≤ (√2−1)/(√2+1)`),
/// from `mpmath.chebyfit(lambda u: 2*log2(e)*atanh(√u)/√u, [0, 0.0295], 13)`.
/// With `t = (m−1)/(m+1)`, `t·poly(t²) = 2·log₂e·atanh(t) = log₂ m`.
pub const LOG2_CH: [DoubleDouble; 13] = [
    DoubleDouble {
        high: 2.885_390_081_777_926_8,
        low: 4.071_054_748_190_96e-17,
    },
    DoubleDouble {
        high: 0.961_796_693_925_975_6,
        low: 5.057_761_610_242_735e-17,
    },
    DoubleDouble {
        high: 0.577_078_016_355_585_3,
        low: 5.255_206_816_760_004e-17,
    },
    DoubleDouble {
        high: 0.412_198_583_111_132_4,
        low: 1.297_102_661_193_637e-17,
    },
    DoubleDouble {
        high: 0.320_598_897_975_325_5,
        low: 2.130_811_880_420_2e-17,
    },
    DoubleDouble {
        high: 0.262_308_189_252_469_5,
        low: -1.750_565_082_034_625_2e-17,
    },
    DoubleDouble {
        high: 0.221_953_083_223_936_92,
        low: 4.474_037_836_742_117e-18,
    },
    DoubleDouble {
        high: 0.192_359_337_770_499,
        low: 9.557_372_598_364_56e-18,
    },
    DoubleDouble {
        high: 0.169_728_896_982_909_97,
        low: -6.916_940_455_706_479e-18,
    },
    DoubleDouble {
        high: 0.151_859_455_190_334_92,
        low: 4.861_603_388_377_106_6e-18,
    },
    DoubleDouble {
        high: 0.137_498_665_102_738_8,
        low: 9.912_843_223_531_668e-18,
    },
    DoubleDouble {
        high: 0.123_472_061_446_308_82,
        low: -3.215_256_324_452_949e-18,
    },
    DoubleDouble {
        high: 0.138_054_266_219_121_37,
        low: 1.199_842_934_052_410_4e-18,
    },
];

/// Double-double minimax of `2ʰ` on `h ∈ [−½, ½]`, low-degree first
///
/// Degree 17, max error `2⁻⁹⁷`, from
/// `mpmath.chebyfit(lambda h: 2**h, [-0.5, 0.5], 18)`.  Reconstructs `2^(E−n)`
/// for [`exp2_dd`]'s fractional part.
pub const EXP2_CE: [DoubleDouble; 18] = [
    DoubleDouble {
        high: 1.0,
        low: 6.209_907_774_086_482e-30,
    },
    DoubleDouble {
        high: 0.693_147_180_559_945_3,
        low: 2.319_046_813_846_322_4e-17,
    },
    DoubleDouble {
        high: 0.240_226_506_959_100_72,
        low: -9.493_931_257_206_889e-18,
    },
    DoubleDouble {
        high: 0.055_504_108_664_821_58,
        low: -3.165_822_290_538_062e-18,
    },
    DoubleDouble {
        high: 0.009_618_129_107_628_477,
        low: 2.832_464_970_675_488_7e-19,
    },
    DoubleDouble {
        high: 0.001_333_355_814_642_844_3,
        low: 1.392_807_521_976_542_5e-20,
    },
    DoubleDouble {
        high: 0.000_154_035_303_933_816_1,
        low: 1.176_599_198_536_661_4e-20,
    },
    DoubleDouble {
        high: 1.525_273_380_405_984_1e-5,
        low: -8.033_876_285_725_231e-22,
    },
    DoubleDouble {
        high: 1.321_548_679_014_431_4e-6,
        low: -6.294_251_571_325_587e-23,
    },
    DoubleDouble {
        high: 1.017_808_600_923_970_1e-7,
        low: -1.958_314_405_806_877e-24,
    },
    DoubleDouble {
        high: 7.054_911_620_796_934e-9,
        low: -9.000_114_207_730_101e-26,
    },
    DoubleDouble {
        high: 4.445_538_271_869_283_4e-10,
        low: -1.937_187_028_252_720_2e-26,
    },
    DoubleDouble {
        high: 2.567_843_602_192_540_5e-11,
        low: -8.396_956_347_010_14e-28,
    },
    DoubleDouble {
        high: 1.369_148_886_427_720_5e-12,
        low: -4.975_429_677_528_511e-29,
    },
    DoubleDouble {
        high: 6.778_715_106_441_096e-14,
        low: -4.426_187_857_196_909e-30,
    },
    DoubleDouble {
        high: 3.132_432_603_948_936_5e-15,
        low: 1.688_398_791_043_837e-32,
    },
    DoubleDouble {
        high: 1.359_423_794_536_793_7e-16,
        low: 1.070_334_018_088_267_4e-32,
    },
    DoubleDouble {
        high: 5.541_797_734_736_37e-18,
        low: 7.021_004_796_570_275e-35,
    },
];

/// `f64` minimax of `2·log₂e·atanh(t)/t` in `u = t²` — the fast path's log
///
/// Degree 10; the `f64` evaluation lands near `2⁻⁵²` relative, ample for the
/// fast path since the Ziv gate catches whatever the single rounding leaves
/// ambiguous.  Same shape as [`LOG2_CH`] without the double-double tail.
pub const LOG2_FAST: [f64; 11] = [
    2.885_390_081_777_926_8,
    0.961_796_693_925_975_6,
    0.577_078_016_355_585_4,
    0.412_198_583_111_126_5,
    0.320_598_897_976_929,
    0.262_308_188_998_910_2,
    0.221_953_108_190_537_57,
    0.192_357_761_894_813_88,
    0.169_792_595_323_944_22,
    0.150_270_565_936_578_03,
    0.159_545_555_181_741_27,
];

/// `f64` minimax of `2ʰ` on `h ∈ [−½, ½]` — the fast path's exp
///
/// Degree 10, error `2⁻⁵²`; counterpart to [`EXP2_CE`] without the low words.
pub const EXP2_FAST: [f64; 11] = [
    1.0,
    0.693_147_180_559_95,
    0.240_226_506_959_100_97,
    0.055_504_108_664_447_72,
    0.009_618_129_107_606_888,
    0.001_333_355_823_016_497_4,
    0.000_154_035_304_417_360_5,
    1.525_265_726_020_083_7e-5,
    1.321_544_258_792_169e-6,
    1.020_869_029_995_830_6e-7,
    7.072_585_949_269_223e-9,
];

/// Evaluate a double-double polynomial `Σ coeffs[k]·uᵏ` at the double-double `u`
///
/// Estrin scheme in double-double arithmetic; the constant term `coeffs[0]` is
/// first (low-degree first).  Used wherever `f64` precision is too coarse — the
/// `log₂ → ×y → exp2` chain of `xʸ`, the `erf`/`erfc` minimax `Q`, and the trig,
/// `atan`, and `gamma` kernels — so its latency dominates those functions.
///
/// Pairing adjacent coefficients into `c[2i] + power·c[2i+1]` and squaring
/// `power` (`u → u² → u⁴ → …`) builds a balanced tree of depth `⌈log₂ n⌉` instead
/// of Horner's length-`n` chain.  Double-double FMAs have a long latency, and the
/// callers evaluate one polynomial per call on the critical path, so shortening
/// the dependency chain — not the operation count — is what speeds them up.
#[inline]
pub fn poly_dd(u: DoubleDouble, coeffs: &[DoubleDouble]) -> DoubleDouble {
    /// Scratch capacity; the largest caller (`erfc_far_eval` accurate path) has 35 terms.
    const CAP: usize = 36;
    const ZERO: DoubleDouble = DoubleDouble {
        high: 0.0,
        low: 0.0,
    };

    let n = coeffs.len();
    debug_assert!(n <= CAP);
    let mut buf = [ZERO; CAP];
    buf[..n].copy_from_slice(coeffs);

    // Each pass folds `buf` (a polynomial in `power`) to half its length and
    // squares `power`.  Writing `buf[i]` only reads `buf[2i]`/`buf[2i+1]`, both
    // at indices `> i` once `i ≥ 1`, so the in-place update never clobbers an
    // unread entry.
    let mut len = n;
    let mut power = u;
    while len > 1 {
        let half = len.div_ceil(2);
        for i in 0..half {
            buf[i] = if 2 * i + 1 < len {
                buf[2 * i] + power * buf[2 * i + 1]
            } else {
                buf[2 * i]
            };
        }
        len = half;
        power = power * power;
    }
    buf[0]
}

/// `log₂(x)` as a double-double for a finite positive `f64`
///
/// Centres the significand on `√2/2` so `x = 2ᵉ·m` with `m ∈ [√2/2, √2)` and
/// `t = (m−1)/(m+1)` stays in `[−0.172, 0.172]`; then `log₂ m = t·poly(t²)` via
/// [`LOG2_CH`] and `log₂ x = e + log₂ m`, all in double-double.
#[inline]
pub fn log2_dd(x: f64) -> DoubleDouble {
    // Normalize a subnormal `x` (×2⁵⁴, exact) so the bit-level exponent extraction
    // is valid, compensating with `extra` in the final exponent.  The f32 `powf`
    // only ever passes normal f64, so `extra` stays 0 there.
    #[allow(clippy::cast_possible_wrap)]
    let (i, extra) = if x < f64::MIN_POSITIVE {
        ((x * crate::exp2i(54)).to_bits() as i64, -54_i64)
    } else {
        (x.to_bits() as i64, 0)
    };
    let exponent = (i - FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let m = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);

    // t = (m−1)/(m+1).  `m − 1` is exact (Sterbenz, `m ∈ [√2/2, √2) ⊂ [0.5, 2)`),
    // but `m + 1` is *not*, so carry it as a double-double `1 + m` and divide in
    // double-double — otherwise the denominator's ≈2⁻⁵³ rounding would dominate the
    // f64 result (it is harmless to the f32 `powf`, which is why it surfaced here).
    let t = DoubleDouble {
        high: m - 1.0,
        low: 0.0,
    } * DoubleDouble::from_sum(1.0, m).recip();
    let log2_m = t * poly_dd(t * t, &LOG2_CH);

    #[allow(clippy::cast_precision_loss)]
    (DoubleDouble {
        high: (exponent + extra) as f64,
        low: 0.0,
    } + log2_m)
}

/// `log₂(x)` as a plain `f64` for the fast path
#[inline]
pub fn log2_fast_path(x: f64) -> f64 {
    #[allow(clippy::cast_possible_wrap)]
    let i = x.to_bits() as i64;
    let exponent = (i - FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let m = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);
    let t = (m - 1.0) / (m + 1.0);

    #[allow(clippy::cast_precision_loss)]
    crate::fast_mul_add(t, crate::poly(t * t, &LOG2_FAST), exponent as f64)
}

/// `log₂(e)` as a double-double, so `log₂x = ln x · log₂e`.
const LOG2_E: DoubleDouble = DoubleDouble {
    high: 1.4426950408889634,
    low: 2.0355273740931033e-17,
};

/// Ziv-gate unit for the [`pow`] fast path, scaled by `1 + |y|`.
///
/// The fast mantissa's absolute error is `≲ (1 + |y|)·2⁻⁶⁵·⁵`: the lean table
/// `exp2` contributes ≈2⁻⁶⁷, and the `×y` amplification of
/// [`ln_fast`](super::ln_fast)'s <2⁻⁶⁶ absolute slip in `log₂x` contributes
/// `log₂e·2⁻⁶⁶·|y| ≈ 2⁻⁶⁵·⁵·|y|` — the `log₂e` and `ln2` factors cancel exactly
/// across the `log₂ → ×y → exp2` round trip.  `2⁻⁶⁴` keeps a ≈2.8× margin over
/// that worst-case bound (the `ln_fast` term is the worst cell with every
/// rounding aligned) while still gating the fast result in for the vast
/// majority of inputs.
const POWF_ZIV_UNIT: f64 = 5.421010862427522e-20; // 2^-64

/// Fast path for [`pow_core`].
///
/// `2^(y·log₂x)` through the lean `ln_fast`→`log₂e`→table-`exp2` chain — the same
/// kernels that make `log2` and `exp2` fast — accepted by a Ziv gate.  `slack`
/// bounds the error of `e = y·log₂x` (and of the [1, 2) mantissa) by
/// `(1 + |y|)·2⁻⁶⁴`, so gross overflow / underflow is decided here directly and
/// only the narrow boundary bands, the subnormal range, and the rare Ziv straddles
/// return `None` for the accurate path.
#[inline]
fn pow_fast(x: f64, y: f64) -> Option<f64> {
    // e = y·log₂x = y · (ln x · log₂e), with `ln_fast` good to <2⁻⁶⁶ absolute, so
    // `e`'s absolute error is below `slack`.
    let e = super::ln_fast(x) * LOG2_E * y;
    let slack = (1.0 + y.abs()) * POWF_ZIV_UNIT;

    // Gross overflow / underflow, decided directly (the `ln_fast` chain already
    // pins `e` more tightly than `log2_dd` would for these out-of-range inputs).
    // The overflow threshold is `2^1025`, not `2^1024`: a result `xʸ` with
    // `log₂(xʸ)` a hair below 1024 still rounds to the finite `DBL_MAX`, and the
    // fast `e.high` can land just above `1024.0` for such an input — so the whole
    // `[1023, 1025)` band is deferred to the accurate path (which rounds the
    // `DBL_MAX`/∞ boundary exactly), and only `e ≥ 2^1025` is certainly ∞ here.
    if e.high - slack > 1025.0 {
        return Some(f64::INFINITY);
    }
    if e.high + slack < -1075.0 {
        return Some(0.0);
    }

    // Leave the over/underflow boundary bands and the normal/subnormal transition
    // to the accurate path, which rounds them exactly.
    if !(e.high + slack < 1023.0 && e.high - slack > -1022.0) {
        return None;
    }

    let (j, q, r) = super::exp::exp2_reduce_dd(e);
    let (m, q) = super::exp::exp_mantissa_fast(j, q, r);

    // Ziv test on the [1, 2) mantissa: accept when both ends of its `±slack` error
    // interval round to the same f64, and the result is comfortably normal.
    let lo = m.high + (m.low - slack);
    let hi = m.high + (m.low + slack);
    (lo == hi && (-1021..=1022).contains(&q)).then(|| fast_ldexp(lo, q))
}

/// Test-only: the fast leg's pre-gate `[1, 2)` mantissa and its `slack` bound, for
/// the [`ziv_soundness`] audit.  Returns `None` when `pow_fast` would defer before
/// forming the mantissa (out-of-range / boundary bands), so the test only checks
/// inputs the gate actually rules on.
#[cfg(all(test, feature = "mpfr"))]
fn powf_fast_leg(x: f64, y: f64) -> Option<(DoubleDouble, i64, f64)> {
    let e = super::ln_fast(x) * LOG2_E * y;
    let slack = (1.0 + y.abs()) * POWF_ZIV_UNIT;
    if e.high - slack > 1024.0 || e.high + slack < -1075.0 {
        return None;
    }
    if !(e.high + slack < 1023.0 && e.high - slack > -1022.0) {
        return None;
    }
    let (j, q, r) = super::exp::exp2_reduce_dd(e);
    let (m, q) = super::exp::exp_mantissa_fast(j, q, r);
    Some((m, q, slack))
}

/// `xʸ` for finite positive `x ≠ 1`, correctly rounded.
///
/// Fast path: [`pow_fast`], a lean `2^(y·log₂x)` accepted by a Ziv gate.  The rare
/// straddles (and the over/underflow / subnormal edges) fall back to
/// [`pow_accurate`](super::pow_accurate::pow_accurate) — a faithful port of
/// CORE-MATH's two final Ziv iterations (a 128-bit `Dint` chain ≈2⁻¹¹³, then a
/// 256-bit `Qint` chain ≈2⁻²⁴⁰) framed by CORE-MATH's exact/midpoint detector,
/// which resolves the rational-exponent ties-to-even cases (`x^1.5 = s³` when
/// `x = s²`, …) that no finite-precision approximation of `xʸ` can decide.
#[inline]
fn pow_core(x: f64, y: f64) -> f64 {
    if let Some(result) = pow_fast(x, y) {
        return result;
    }

    // Infinite exponent: `pow_fast` defers it (its `slack` becomes ∞), and the
    // accurate path cannot represent it.  With `x` finite, positive and ≠ 1,
    // `e = y·log₂x = ±∞` with sign `sign(y)·sign(x−1)`, so the limit is ∞ or 0.
    if !y.is_finite() {
        return if (y > 0.0) == (x > 1.0) {
            f64::INFINITY
        } else {
            0.0
        };
    }

    // `x` is the positive magnitude here; the negative-base sign fold lives in
    // [`pow`].  So the accurate path computes the positive magnitude (`s = +1`)
    // and `x0 = x`.
    super::pow_accurate::pow_accurate(x, y, x, 1.0)
}

/// Raise to a floating-point power
#[must_use]
#[inline]
pub fn pow(x: f64, y: f64) -> f64 {
    #[inline]
    fn magnitude(x: f64, y: f64) -> f64 {
        match x.classify() {
            FpCategory::Nan => f64::NAN,
            FpCategory::Infinite => match y.partial_cmp(&0.0) {
                Some(Ordering::Greater) => f64::INFINITY,
                Some(Ordering::Less) => 0.0,
                Some(Ordering::Equal) => 1.0,
                None => f64::NAN,
            },
            FpCategory::Zero => match y.partial_cmp(&0.0) {
                Some(Ordering::Greater) => 0.0,
                Some(Ordering::Less) => f64::INFINITY,
                Some(Ordering::Equal) => 1.0,
                None => f64::NAN,
            },
            // Normal or subnormal base.
            _ => {
                if x == 1.0 {
                    1.0 // 1ʸ = 1 for every y, including NaN
                } else if x.is_sign_negative() || y.is_nan() {
                    f64::NAN // negative base (complex result) or NaN exponent, x ≠ 1
                } else {
                    // xʸ = 2^(y·log₂x): lean fast leg, CORE-MATH accurate finisher.
                    pow_core(x, y)
                }
            }
        }
    }

    #[inline]
    fn is_integer(x: f64) -> bool {
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

/// Fast path for [`compound_core`] — [`pow_fast`] with a `1+x` front end.
///
/// `2^(y·log₂(1+x))` through the lean chain, with `log₂(1+x)` formed as
/// `ln(1+x)·log₂e`: the exact `1+x` double-double [`from_sum`](DoubleDouble::from_sum)
/// (a 2Sum, exact for every finite `x`) feeds [`ln_dd_fast`](super::ln_dd_fast),
/// whose reduction recovers the `−x²/2 …` terms that a raw `ln_fast(1+x)` would
/// lose when `1+x` rounds.  Everything after `e = log₂(1+x)·y` is identical to
/// [`pow_fast`]: the same `(1 + |y|)·2⁻⁶⁴` slack, gross over/underflow decision,
/// boundary-band defer, and Ziv gate on the `[1, 2)` mantissa.  A subnormal
/// `1 + x` (`x` within ~2⁻¹⁰²² of −1) defers to the accurate path.
#[inline]
fn compound_fast(x: f64, y: f64) -> Option<f64> {
    let s = DoubleDouble::from_sum(1.0, x);
    if !(s.high >= f64::MIN_POSITIVE) {
        return None; // 1 + x subnormal / zero: let the accurate path round it
    }

    // Renormalize `ln_dd_fast`'s raw pair (it skips the closing Fast2Sum that
    // `ln_fast` applies) so the `× LOG2_E × y` chain matches `pow_fast`'s accuracy.
    let l = super::ln_dd_fast(s);
    let e = fast_sum(l.high, l.low) * LOG2_E * y;
    let slack = (1.0 + y.abs()) * POWF_ZIV_UNIT;

    if e.high - slack > 1025.0 {
        return Some(f64::INFINITY);
    }
    if e.high + slack < -1075.0 {
        return Some(0.0);
    }

    if !(e.high + slack < 1023.0 && e.high - slack > -1022.0) {
        return None;
    }

    let (j, q, r) = super::exp::exp2_reduce_dd(e);
    let (m, q) = super::exp::exp_mantissa_fast(j, q, r);

    let lo = m.high + (m.low - slack);
    let hi = m.high + (m.low + slack);
    (lo == hi && (-1021..=1022).contains(&q)).then(|| fast_ldexp(lo, q))
}

/// Test-only counterpart of [`powf_fast_leg`] for [`compound_fast`]: the pre-gate
/// `[1, 2)` mantissa and its `slack`, for the [`ziv_soundness`] audit.
#[cfg(all(test, feature = "mpfr"))]
fn compound_fast_leg(x: f64, y: f64) -> Option<(DoubleDouble, i64, f64)> {
    let s = DoubleDouble::from_sum(1.0, x);
    if !(s.high >= f64::MIN_POSITIVE) {
        return None;
    }
    let l = super::ln_dd_fast(s);
    let e = fast_sum(l.high, l.low) * LOG2_E * y;
    let slack = (1.0 + y.abs()) * POWF_ZIV_UNIT;
    if e.high - slack > 1024.0 || e.high + slack < -1075.0 {
        return None;
    }
    if !(e.high + slack < 1023.0 && e.high - slack > -1022.0) {
        return None;
    }
    let (j, q, r) = super::exp::exp2_reduce_dd(e);
    let (m, q) = super::exp::exp_mantissa_fast(j, q, r);
    Some((m, q, slack))
}

/// `(1+x)ʸ` for finite `x > −1`, `x ≠ 0`, finite `y ∉ {0, 1}`, correctly rounded.
///
/// Fast path: [`compound_fast`], a lean `2^(y·log₂(1+x))` accepted by a Ziv gate.
/// The rare straddles (and the over/underflow / subnormal edges) fall back to
/// [`compound_accurate`](super::pow_accurate::compound_accurate) — the same
/// `Dint`/`Qint` cascade and exact/midpoint detector as [`pow_core`], run on the
/// exactly-formed base `1 + x`.  Infinite `y` is handled by [`compound`], so `y`
/// is finite here.
#[inline]
fn compound_core(x: f64, y: f64) -> f64 {
    compound_fast(x, y).unwrap_or_else(|| super::pow_accurate::compound_accurate(x, y))
}

/// Signaling NaN: an `f64` NaN with the quiet bit clear.
#[inline]
fn is_snan(x: f64) -> bool {
    x.is_nan() && x.to_bits() & 0x0008_0000_0000_0000 == 0
}

/// Compound interest: `(1 + x)ʸ`, correctly rounded (C23's `compound`)
///
/// The special-value contract follows C23 F.10.4.1: `compound(±0, y) = 1` for
/// every `y` (even ±∞ and quiet NaN) and `compound(x, ±0) = 1` for every
/// `x ≥ −1` (and quiet-NaN `x`) — a signaling NaN still yields NaN; `x < −1`
/// (including −∞) is a domain error; `compound(−1, y)` is `+0` for `y > 0` and
/// `+∞` for `y < 0`.  The kernel is [`compound_core`]; `y = 1` returns the
/// exactly-representable `1 + x` directly.  The f64 counterpart of the f32
/// [`compoundf`](crate::compoundf).
#[must_use]
#[inline]
pub fn compound(x: f64, y: f64) -> f64 {
    if x == 0.0 {
        return if is_snan(y) { x + y } else { 1.0 };
    }
    if y == 0.0 {
        if is_snan(x) || x < -1.0 {
            return x + f64::NAN;
        }
        return 1.0; // includes x = +∞ and quiet NaN
    }
    if x.is_nan() || y.is_nan() {
        return x + y;
    }
    if x < -1.0 {
        return f64::NAN; // domain: 1 + x < 0, includes −∞
    }
    if y.is_infinite() {
        // (1+x) against 1 decides growth or decay; x = −1 decays too.
        return if (x > 0.0) == (y > 0.0) {
            f64::INFINITY
        } else {
            0.0
        };
    }
    if x == f64::INFINITY {
        return if y > 0.0 { f64::INFINITY } else { 0.0 };
    }
    if x == -1.0 {
        return if y > 0.0 { 0.0 } else { f64::INFINITY };
    }
    if y == 1.0 {
        return 1.0 + x; // exact whenever 1 + x is representable; rounds like the C
    }
    compound_core(x, y)
}

#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::*;
    use rug::Float;
    use rug::ops::Pow;

    const fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// The fast leg's Ziv gate must be *sound*: the `slack = (1+|y|)·2⁻⁶⁴` it tests
    /// the `[1, 2)` mantissa against has to exceed that mantissa's true absolute
    /// error with margin, or a confident `lo == hi` could certify a value on the
    /// wrong side of a rounding boundary.  We sweep the hard regime (`x` near 1,
    /// large `|y|`, where the `×y` amplification is worst) and the wide regime,
    /// measure `|m − x^y/2^q| / slack`, and require it stay below ½ (i.e. slack is
    /// ≥ 2× the true error).
    #[test]
    fn powf_fast_leg_is_sound() {
        let mut worst = 0.0_f64;
        let mut worst_xy = (0.0, 0.0);
        let mut n_checked = 0u64;

        for i in 0..6_000_000u64 {
            // Hard regime: x ∈ [1, 2) hashed, y value-uniform in [−1100, 1100].
            let (x, y) = if i & 1 == 0 {
                let xb = 0x3FF0_0000_0000_0000 | (mix(i) >> 12);
                let yu = (mix(i ^ 0xABCD) >> 11) as f64 / (1u64 << 53) as f64;
                (f64::from_bits(xb), crate::fma(yu, 2200.0, -1100.0))
            } else {
                // Wide regime: any positive x, y in [−40, 40].
                let xb = (mix(i) & 0x7FFF_FFFF_FFFF_FFFF) | 1;
                let yu = (mix(i ^ 0x1234) >> 11) as f64 / (1u64 << 53) as f64;
                (f64::from_bits(xb), crate::fma(yu, 80.0, -40.0))
            };
            if !(x.is_finite() && x > 0.0 && x != 1.0) {
                continue;
            }
            let Some((m, q, slack)) = powf_fast_leg(x, y) else {
                continue;
            };
            // True [1, 2) mantissa = x^y / 2^q at 250 bits.
            let truth = Float::with_val(250, x).pow(Float::with_val(250, y))
                / Float::with_val(250, 2).pow(q);
            if truth == 0.0 || !truth.is_finite() {
                continue;
            }
            let got = Float::with_val(250, m.high) + Float::with_val(250, m.low);
            let abs_err = Float::with_val(250, &got - &truth).abs().to_f64();
            let ratio = abs_err / slack;
            if ratio > worst {
                worst = ratio;
                worst_xy = (x, y);
            }
            n_checked += 1;
        }

        println!(
            "powf_fast_leg: worst |error|/slack = {worst:.4} over {n_checked} gated inputs \
             (at x={:e}, y={:e}); sound iff < 0.5",
            worst_xy.0, worst_xy.1
        );
        assert!(
            worst < 0.5,
            "POWF_ZIV_UNIT unsound: fast-leg error reaches {worst:.4}× slack \
             (need < 0.5 for a 2× margin) at x={:e}, y={:e}",
            worst_xy.0,
            worst_xy.1
        );
    }

    /// The [`compound_fast`] Ziv gate reuses `pow`'s `slack = (1+|y|)·2⁻⁶⁴`, so
    /// its `[1, 2)` mantissa must stay within that of `(1+x)ʸ`'s truth by ≥ 2×.
    /// The hard regime is `1 + x` near 1 (`|x|` small, where `log₂(1+x)` is small
    /// and the `×y` amplification is largest); we also sweep a wide regime.
    #[test]
    fn compound_fast_leg_is_sound() {
        let mut worst = 0.0_f64;
        let mut worst_xy = (0.0, 0.0);
        let mut n_checked = 0u64;

        for i in 0..6_000_000u64 {
            let (x, y) = if i & 1 == 0 {
                // Hard regime: base near 1 (|x| ∈ [2⁻⁵⁵, 1)), large |y|.
                let u = (mix(i) >> 11) as f64 / (1u64 << 53) as f64;
                let s = crate::fma(u, 2.0, -1.0); // [−1, 1)
                let scale = crate::exp2i(-(1 + (mix(i ^ 0x5555) % 55) as i64));
                let x = s * scale;
                let yu = (mix(i ^ 0xABCD) >> 11) as f64 / (1u64 << 53) as f64;
                (x, crate::fma(yu, 2200.0, -1100.0))
            } else {
                // Wide regime: x anywhere in (−1, 40], moderate y.
                let xu = (mix(i) >> 11) as f64 / (1u64 << 53) as f64;
                let x = crate::fma(xu, 41.0, -0.999);
                let yu = (mix(i ^ 0x1234) >> 11) as f64 / (1u64 << 53) as f64;
                (x, crate::fma(yu, 80.0, -40.0))
            };
            if !(x.is_finite() && x > -1.0 && x != 0.0) {
                continue;
            }
            let Some((m, q, slack)) = compound_fast_leg(x, y) else {
                continue;
            };
            // True [1, 2) mantissa = (1+x)^y / 2^q at 250 bits.
            let base = Float::with_val(250, x) + 1_u32;
            let truth = base.pow(Float::with_val(250, y)) / Float::with_val(250, 2).pow(q);
            if truth == 0.0 || !truth.is_finite() {
                continue;
            }
            let got = Float::with_val(250, m.high) + Float::with_val(250, m.low);
            let abs_err = Float::with_val(250, &got - &truth).abs().to_f64();
            let ratio = abs_err / slack;
            if ratio > worst {
                worst = ratio;
                worst_xy = (x, y);
            }
            n_checked += 1;
        }

        println!(
            "compound_fast_leg: worst |error|/slack = {worst:.4} over {n_checked} gated inputs \
             (at x={:e}, y={:e}); sound iff < 0.5",
            worst_xy.0, worst_xy.1
        );
        assert!(
            worst < 0.5,
            "compound fast-leg unsound: error reaches {worst:.4}× slack \
             (need < 0.5 for a 2× margin) at x={:e}, y={:e}",
            worst_xy.0,
            worst_xy.1
        );
    }
}
