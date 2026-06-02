//! Power function `xʸ = 2^(y·log₂x)`, correctly rounded.
//!
//! The double-double `log₂`/`exp2` kernels here are shared with the f32
//! [`powf`](crate::f32::powf): they already run in f64 double-double, since even
//! for an f32 result the error in `log₂x` is amplified by `y`.  This module hosts
//! them (`pub(crate)`) and adds the f64-output `exp2_dd`/`powf_core`.
#![allow(clippy::unreadable_literal, clippy::excessive_precision)]

use super::double::{round_general64, Sum};
use super::EXP_SHIFT;
use core::cmp::Ordering;
use core::f64::consts::FRAC_1_SQRT_2;
use core::num::FpCategory;

/// Double-double minimax of `2·log₂e·atanh(t)/t` in `u = t²`, low-degree first
///
/// Degree 12, max error `2⁻⁹⁴` on `u ∈ [0, 0.0295]` (i.e. `|t| ≤ (√2−1)/(√2+1)`),
/// from `mpmath.chebyfit(lambda u: 2*log2(e)*atanh(√u)/√u, [0, 0.0295], 13)`.
/// With `t = (m−1)/(m+1)`, `t·poly(t²) = 2·log₂e·atanh(t) = log₂ m`.
pub(crate) const LOG2_CH: [Sum; 13] = [
    Sum {
        high: 2.885_390_081_777_926_8,
        low: 4.071_054_748_190_96e-17,
    },
    Sum {
        high: 0.961_796_693_925_975_6,
        low: 5.057_761_610_242_735e-17,
    },
    Sum {
        high: 0.577_078_016_355_585_3,
        low: 5.255_206_816_760_004e-17,
    },
    Sum {
        high: 0.412_198_583_111_132_4,
        low: 1.297_102_661_193_637e-17,
    },
    Sum {
        high: 0.320_598_897_975_325_5,
        low: 2.130_811_880_420_2e-17,
    },
    Sum {
        high: 0.262_308_189_252_469_5,
        low: -1.750_565_082_034_625_2e-17,
    },
    Sum {
        high: 0.221_953_083_223_936_92,
        low: 4.474_037_836_742_117e-18,
    },
    Sum {
        high: 0.192_359_337_770_499,
        low: 9.557_372_598_364_56e-18,
    },
    Sum {
        high: 0.169_728_896_982_909_97,
        low: -6.916_940_455_706_479e-18,
    },
    Sum {
        high: 0.151_859_455_190_334_92,
        low: 4.861_603_388_377_106_6e-18,
    },
    Sum {
        high: 0.137_498_665_102_738_8,
        low: 9.912_843_223_531_668e-18,
    },
    Sum {
        high: 0.123_472_061_446_308_82,
        low: -3.215_256_324_452_949e-18,
    },
    Sum {
        high: 0.138_054_266_219_121_37,
        low: 1.199_842_934_052_410_4e-18,
    },
];

/// Double-double minimax of `2ʰ` on `h ∈ [−½, ½]`, low-degree first
///
/// Degree 17, max error `2⁻⁹⁷`, from
/// `mpmath.chebyfit(lambda h: 2**h, [-0.5, 0.5], 18)`.  Reconstructs `2^(E−n)`
/// for [`exp2_dd`]'s fractional part.
pub(crate) const EXP2_CE: [Sum; 18] = [
    Sum {
        high: 1.0,
        low: 6.209_907_774_086_482e-30,
    },
    Sum {
        high: 0.693_147_180_559_945_3,
        low: 2.319_046_813_846_322_4e-17,
    },
    Sum {
        high: 0.240_226_506_959_100_72,
        low: -9.493_931_257_206_889e-18,
    },
    Sum {
        high: 0.055_504_108_664_821_58,
        low: -3.165_822_290_538_062e-18,
    },
    Sum {
        high: 0.009_618_129_107_628_477,
        low: 2.832_464_970_675_488_7e-19,
    },
    Sum {
        high: 0.001_333_355_814_642_844_3,
        low: 1.392_807_521_976_542_5e-20,
    },
    Sum {
        high: 0.000_154_035_303_933_816_1,
        low: 1.176_599_198_536_661_4e-20,
    },
    Sum {
        high: 1.525_273_380_405_984_1e-5,
        low: -8.033_876_285_725_231e-22,
    },
    Sum {
        high: 1.321_548_679_014_431_4e-6,
        low: -6.294_251_571_325_587e-23,
    },
    Sum {
        high: 1.017_808_600_923_970_1e-7,
        low: -1.958_314_405_806_877e-24,
    },
    Sum {
        high: 7.054_911_620_796_934e-9,
        low: -9.000_114_207_730_101e-26,
    },
    Sum {
        high: 4.445_538_271_869_283_4e-10,
        low: -1.937_187_028_252_720_2e-26,
    },
    Sum {
        high: 2.567_843_602_192_540_5e-11,
        low: -8.396_956_347_010_14e-28,
    },
    Sum {
        high: 1.369_148_886_427_720_5e-12,
        low: -4.975_429_677_528_511e-29,
    },
    Sum {
        high: 6.778_715_106_441_096e-14,
        low: -4.426_187_857_196_909e-30,
    },
    Sum {
        high: 3.132_432_603_948_936_5e-15,
        low: 1.688_398_791_043_837e-32,
    },
    Sum {
        high: 1.359_423_794_536_793_7e-16,
        low: 1.070_334_018_088_267_4e-32,
    },
    Sum {
        high: 5.541_797_734_736_37e-18,
        low: 7.021_004_796_570_275e-35,
    },
];

/// `f64` minimax of `2·log₂e·atanh(t)/t` in `u = t²` — the fast path's log
///
/// Degree 10; the `f64` evaluation lands near `2⁻⁵²` relative, ample for the
/// fast path since the Ziv gate catches whatever the single rounding leaves
/// ambiguous.  Same shape as [`LOG2_CH`] without the double-double tail.
pub(crate) const LOG2_FAST: [f64; 11] = [
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
pub(crate) const EXP2_FAST: [f64; 11] = [
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
/// Horner in double-double arithmetic; the leading coefficient must be last.
/// Used by [`log2_dd`]/[`exp2_dd`], where `f64` precision is far too coarse: in
/// `xʸ = 2^(y·log₂x)` the error in `log₂x` is amplified by `y`, so the whole
/// `log₂ → ×y → exp2` chain runs in double-double.
#[inline]
pub(crate) fn poly_dd(u: Sum, coeffs: &[Sum]) -> Sum {
    let (last, rest) = coeffs.split_last().unwrap();
    let mut acc = *last;
    for c in rest.iter().rev() {
        acc = acc * u + *c;
    }
    acc
}

/// `log₂(x)` as a double-double for a finite positive `f64`
///
/// Centres the significand on `√2/2` so `x = 2ᵉ·m` with `m ∈ [√2/2, √2)` and
/// `t = (m−1)/(m+1)` stays in `[−0.172, 0.172]`; then `log₂ m = t·poly(t²)` via
/// [`LOG2_CH`] and `log₂ x = e + log₂ m`, all in double-double.
#[inline]
pub(crate) fn log2_dd(x: f64) -> Sum {
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
    let t = Sum {
        high: m - 1.0,
        low: 0.0,
    } * Sum::from_sum(1.0, m).recip();
    let log2_m = t * poly_dd(t * t, &LOG2_CH);

    #[allow(clippy::cast_precision_loss)]
    (Sum {
        high: (exponent + extra) as f64,
        low: 0.0,
    } + log2_m)
}

/// `log₂(x)` as a plain `f64` for the fast path
#[inline]
pub(crate) fn log2_fast_path(x: f64) -> f64 {
    #[allow(clippy::cast_possible_wrap)]
    let i = x.to_bits() as i64;
    let exponent = (i - FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let m = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);
    let t = (m - 1.0) / (m + 1.0);

    #[allow(clippy::cast_precision_loss)]
    crate::mul_add(t, crate::poly(t * t, &LOG2_FAST), exponent as f64)
}

/// `2^e` correctly rounded to `f64`, taking a double-double exponent
///
/// Splits `e = n + h` with `n = round(e)` and `|h| ≤ ½`, evaluates `2ʰ` in
/// double-double via [`EXP2_CE`], normalizes the mantissa into `[1, 2)`, and
/// rounds `mantissa · 2ⁿ` with [`round_general64`] (subnormal-safe).
#[inline]
fn exp2_dd(e: Sum) -> f64 {
    // `2^1024` overflows and `2^−1075` rounds to zero; everything between is
    // resolved by the rounder.
    if e.high > 1024.0 {
        return f64::INFINITY;
    }
    if e.high < -1075.0 {
        return 0.0;
    }

    let n = e.high.round_ties_even();
    let h = Sum::from_sum(e.high - n, e.low);
    let m = poly_dd(h, &EXP2_CE);

    // SAFETY: `−1075 ≤ e.high ≤ 1024` bounds `n` well within `i64`.
    let mut n = unsafe { n.to_int_unchecked::<i64>() };

    // `m = 2ʰ ∈ [√2/2, √2)`; bring it into [1, 2) for the rounder.
    let m = if m.high < 1.0 {
        n -= 1;
        m * 2.0
    } else {
        m
    };

    // `m ∈ [1, 2)`, so the result is `≥ 2ⁿ`; `n ≥ 1024` means overflow.
    if n >= 1024 {
        return f64::INFINITY;
    }
    round_general64(m, n)
}

/// `xʸ` for finite positive `x ≠ 1`, correctly rounded to `f64`
///
/// `2^(y·log₂x)` evaluated entirely in double-double: [`log2_dd`] → `×y` →
/// [`exp2_dd`].  Unlike the f32 [`powf`](crate::f32::powf), there is no plain-f64
/// fast path — an f64 fast result carries no extra precision to Ziv-test against,
/// so the double-double chain (good to ≈2⁻⁸⁴ after the `×y` amplification) is the
/// only path.  Overflow / underflow are handled inside [`exp2_dd`].
#[inline]
fn powf_core(x: f64, y: f64) -> f64 {
    exp2_dd(log2_dd(x) * y)
}

/// Raise to a floating-point power
#[must_use]
#[inline]
pub fn powf(x: f64, y: f64) -> f64 {
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
                } else if x.is_sign_negative() {
                    f64::NAN
                } else if y.is_nan() {
                    f64::NAN // xⁿᵃⁿ = NaN for x ≠ 1
                } else {
                    // xʸ = 2^(y·log₂x), entirely in double-double.
                    powf_core(x, y)
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
