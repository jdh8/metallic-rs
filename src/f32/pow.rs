use crate::f64::double::{fast_ldexp, Sum};
use core::cmp::Ordering;
use core::num::FpCategory;

/// Double-double minimax of `2·log₂e·atanh(t)/t` in `u = t²`, low-degree first
///
/// Degree 12, max error `2⁻⁹⁴` on `u ∈ [0, 0.0295]` (i.e. `|t| ≤ (√2−1)/(√2+1)`),
/// from `mpmath.chebyfit(lambda u: 2*log2(e)*atanh(√u)/√u, [0, 0.0295], 13)`.
/// With `t = (m−1)/(m+1)`, `t·poly(t²) = 2·log₂e·atanh(t) = log₂ m`.
const LOG2_CH: [Sum; 13] = [
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
const EXP2_CE: [Sum; 18] = [
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

/// `f64` minimax of `2·log₂e·atanh(t)/t` in `u = t²` — the fast `powf` path's log
///
/// Degree 10; the `f64` evaluation lands near `2⁻⁵²` relative, ample for the
/// fast path since the Ziv gate catches whatever the single rounding leaves
/// ambiguous.  Same shape as [`LOG2_CH`] without the double-double tail.
const LOG2_FAST: [f64; 11] = [
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

/// `f64` minimax of `2ʰ` on `h ∈ [−½, ½]` — the fast `powf` path's exp
///
/// Degree 10, error `2⁻⁵²`; counterpart to [`EXP2_CE`] without the low words.
const EXP2_FAST: [f64; 11] = [
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
/// Used by [`log2_dd`] and [`exp2_dd`], where `f64` precision is far too coarse:
/// in `xʸ = 2^(y·log₂x)` the error in `log₂x` is amplified by `y`, so the whole
/// `log₂ → ×y → exp2` chain runs in double-double.
#[inline]
fn poly_dd(u: Sum, coeffs: &[Sum]) -> Sum {
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
/// [`LOG2_CH`] and `log₂ x = e + log₂ m`, all in double-double.  The heavy kernel
/// powering [`powf`](super::powf).
#[inline]
fn log2_dd(x: f64) -> Sum {
    use crate::f64::EXP_SHIFT;
    use core::f64::consts::FRAC_1_SQRT_2;

    #[allow(clippy::cast_possible_wrap)]
    let i = x.to_bits() as i64;
    let exponent = (i - FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let m = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);
    let t = Sum::from_quotient(m - 1.0, m + 1.0);
    let log2_m = t * poly_dd(t * t, &LOG2_CH);

    #[allow(clippy::cast_precision_loss)]
    (Sum {
        high: exponent as f64,
        low: 0.0,
    } + log2_m)
}

/// `2^e` correctly rounded to `f32`, taking a double-double exponent
///
/// Splits `e = n + h` with `n = round(e)` and `|h| ≤ ½`, evaluates `2ʰ` in
/// double-double via [`EXP2_CE`], scales by `2ⁿ` with [`fast_ldexp`], and rounds
/// the double-double with [`round_general`] (round-to-odd, subnormal-safe).  The
/// argument carries enough precision that the round is correct for [`powf`].
#[inline]
fn exp2_dd(e: Sum) -> f32 {
    use crate::f64::double::round_general;

    if e.high > 130.0 {
        return f32::INFINITY;
    }
    if e.high < -160.0 {
        return 0.0;
    }

    let n = e.high.round_ties_even();
    let h = Sum::from_sum(e.high - n, e.low);
    let m = poly_dd(h, &EXP2_CE);

    // SAFETY: `-160 ≤ e.high ≤ 130` bounds `n`, so the scaling stays in range.
    let n = unsafe { n.to_int_unchecked() };

    round_general(Sum {
        high: fast_ldexp(m.high, n),
        low: fast_ldexp(m.low, n),
    })
}

/// `log₂(x)` as a plain `f64` for the fast [`powf_core`] path
#[inline]
fn log2_fast_path(x: f64) -> f64 {
    use crate::f64::EXP_SHIFT;
    use core::f64::consts::FRAC_1_SQRT_2;

    #[allow(clippy::cast_possible_wrap)]
    let i = x.to_bits() as i64;
    let exponent = (i - FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let m = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);
    let t = (m - 1.0) / (m + 1.0);

    #[allow(clippy::cast_precision_loss)]
    crate::mul_add(t, crate::poly(t * t, &LOG2_FAST), exponent as f64)
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

#[cfg(feature = "core-math")]
pub use core_math::powf;

/// Raise to a floating-point power
#[cfg(not(feature = "core-math"))]
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
