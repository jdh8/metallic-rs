use super::log::{LNF_TABLES, log_lookup};
use crate::f64_::double::{DoubleDouble, fast_ldexp, round_general, round_signed};

/// π as a double-double (needed for lgamma reflection)
const PI: DoubleDouble = DoubleDouble {
    high: 3.141_592_653_589_793,
    low: 1.224_646_799_147_353_2e-16,
};

/// Center of the tgamma minimax interval, `Γ(TGAMMA_CENTER + d)`
const TGAMMA_CENTER: f64 = 2.875;

/// Leading double-double coefficients of `Γ(2.875 + d)`, `c₀..c₅`
///
/// Paired with [`TGAMMA_TAIL`]; together a degree-18 minimax fit on `d ∈ [−½, ½]`
/// with relative approximation error `2⁻⁶⁷`, from
/// `mpmath.chebyfit(lambda d: gamma(mpf("2.875")+d), [-0.5,0.5], 19)` at
/// `prec=260`.  Only the leading six coefficients need double-double — `c_k`'s
/// f64 rounding contributes `2⁻⁵³·|c_k|·½ᵏ`, which drops below `2⁻⁶²` from `c₆`.
const TGAMMA_DD: [DoubleDouble; 6] = [
    DoubleDouble {
        high: 1.787_710_898_896_940_3,
        low: -3.737_560_105_011_311e-17,
    },
    DoubleDouble {
        high: 1.559_193_901_207_950_5,
        low: -6.845_438_926_265_673e-18,
    },
    DoubleDouble {
        high: 1.051_049_326_681_183,
        low: -8.108_633_068_691_18e-17,
    },
    DoubleDouble {
        high: 0.470_658_018_293_397_15,
        low: 1.134_348_917_718_500_7e-18,
    },
    DoubleDouble {
        high: 0.188_818_638_320_115_08,
        low: -3.201_351_369_000_903_3e-18,
    },
    DoubleDouble {
        high: 0.058_831_548_410_609_08,
        low: -4.385_547_942_551_65e-20,
    },
];

/// `f64` tail coefficients `c₆..c₁₈` of `Γ(2.875 + d)`, low-degree first
const TGAMMA_TAIL: [f64; 13] = [
    0.017_825_943_641_179_64,
    0.004_228_758_172_382_065,
    0.001_097_918_031_010_185,
    0.000_194_565_434_866_167_52,
    5.196_979_083_766_004_4e-5,
    4.915_691_029_862_106_5e-6,
    2.444_402_450_877_52e-6,
    -1.498_674_146_532_898e-7,
    1.650_360_438_390_299_6e-7,
    -4.001_740_593_832_342e-8,
    1.641_622_008_561_281_3e-8,
    -6.297_805_420_549_436_6e-9,
    2.238_642_403_291_221_4e-9,
];

/// `Γ(2.875 + d)` as a double-double for `d ∈ [−½, ½]`
///
/// The high-degree tail `c₆ + c₇d + …` is summed in `f64` (its rounding enters at
/// `d⁶`, hence below `2⁻⁶³`) and folded into a double-double Horner pass over the
/// six leading coefficients, giving a relative error near `2⁻⁶⁴` — ample for a
/// correctly-rounded `f32` after the recurrence and round-to-odd.
#[inline]
fn tgamma_poly(d: f64) -> DoubleDouble {
    let tail = crate::poly(d, &TGAMMA_TAIL);

    let mut acc = TGAMMA_DD[5] + DoubleDouble::from_product(d, tail);
    for coefficient in TGAMMA_DD[..5].iter().rev() {
        acc = acc * d + *coefficient;
    }

    acc
}

/// `Γ(2.875 + d)` for `d ∈ [−½, ½]` as a plain-`f64` degree-11 minimax
///
/// Relative error `2⁻⁴²` — the fast Ziv path of [`tgamma`] only needs enough to
/// gate against the double-double [`tgamma_poly`], so half the degree suffices.
const TGAMMA_POLY_F64: [f64; 12] = [
    1.787_710_898_896_633_5,
    1.559_193_901_207_972_3,
    1.051_049_326_769_541_3,
    0.470_658_018_287_150_76,
    0.188_818_634_201_838_25,
    0.058_831_548_700_820_71,
    0.017_826_013_745_994_518,
    0.004_228_753_264_700_664,
    0.001_097_379_867_515_545,
    0.000_194_602_632_143_494_85,
    5.386_339_273_934_268_4e-5,
    4.788_385_422_314_536e-6,
];

/// `½·ln(2π)`, the additive Stirling constant for `lgamma_pos_dd`
const HALF_LN_2PI: DoubleDouble = DoubleDouble {
    high: 0.918_938_533_204_672_8,
    low: -3.878_294_158_067_241_4e-17,
};

/// Argument above which the Stirling series for `ln Γ` converges fast enough
const LGAMMA_STIRLING: f64 = 14.0;

/// Stirling tail `Σ_{k≥2} B₂ₖ/(2k(2k−1)) t^{1−2k}` past the leading `1/(12t)`
///
/// Coefficients `B₄/12`, `B₆/30`, … the Bernoulli factors of `ln Γ`'s asymptotic
/// expansion; evaluated as `(u/t)·poly(u)` with `u = 1/t²`.
const LGAMMA_TAIL: [f64; 7] = [
    -0.002_777_777_777_777_778,
    0.000_793_650_793_650_793_7,
    -0.000_595_238_095_238_095_3,
    0.000_841_750_841_750_841_7,
    -0.001_917_526_917_526_917_6,
    0.006_410_256_410_256_41,
    -0.029_550_653_594_771_242,
];

/// Numerator of the `f64` rational `g(z) = ln Γ(z)/((z−1)(z−2))` on `[½, 8]`
///
/// Degree 7/7 minimax (rminimax), relative error `2⁻³⁷` — only the *fast* Ziv
/// path of [`lgamma`] uses it, so `f64` coefficients are ample.  The `(z−1)(z−2)`
/// factor carries lgamma's zeros at 1 and 2; the rational stays log-free, which
/// is the whole point of the small-argument path.
const LGAMMA_NUM: [f64; 8] = [
    0.006_304_151_792_758_075,
    0.124_171_423_430_961_66,
    0.479_383_552_214_150_1,
    0.555_061_667_914_565_5,
    0.211_973_927_253_084_42,
    0.025_465_952_676_201_734,
    0.000_736_538_147_975_647_5,
    1.113_967_406_563_718_2e-6,
];

/// Denominator of the rational `g(z)` paired with [`LGAMMA_NUM`]
const LGAMMA_DEN: [f64; 8] = [
    0.002_584_416_514_557_812,
    0.090_842_150_680_063_21,
    0.556_476_452_487_686_2,
    1.0,
    0.628_146_845_505_252_6,
    0.142_068_778_652_565_23,
    0.010_505_967_556_747_394,
    0.000_179_700_933_585_125_6,
];

/// `|sin(πx)|`, accurate to ≈2⁻⁵¹ relative — the lgamma reflection's sine factor
///
/// Reduces to the distance to the nearest integer, `r = x − round(x) ∈ [−½, ½]`,
/// where `|sin(πx)| = |r|·P(r²)`.  A single odd polynomial — no quadrant branch
/// and no second cosine series like [`sinpi`] — keeps the `≈π|r|` behaviour exact
/// at the integer poles, where `ln|sin(πx)|` and hence lgamma blow up.  The
/// minimax `P(u) ≈ sin(π√u)/√u` on `u ∈ [0, ¼]` is from
/// `mpmath.chebyfit(lambda u: sin(pi·√u)/√u, [0, 0.25], 8)`.
#[inline]
fn abs_sinpi(x: f32) -> f64 {
    let x = f64::from(x);
    let r = x - x.round_ties_even();

    r.abs()
        * crate::poly(
            r * r,
            &[
                3.141_592_653_589_792_7,
                -5.167_712_780_049_785_5,
                2.550_164_039_861_867_7,
                -0.599_264_528_825_224,
                0.082_145_878_816_310_43,
                -0.007_370_364_326_530_415_5,
                0.000_465_987_015_804_337_8,
                -2.113_362_735_205_297e-5,
            ],
        )
}

/// `sin(πx)`
///
/// The argument is reduced modulo 2 into `[-1, 1]`, then into `[-¼, ¼]` with a
/// quadrant `q`, where `sin(πx)` is one of `±sin(πr)`, `±cos(πr)`.
#[inline]
fn sinpi(x: f32) -> f64 {
    let x = crate::fma(2.0, -(0.5f32 * x).round_ties_even() as f64, x as f64) as f32;
    let q = (2.0 * x).round_ties_even();
    let r = crate::fma(0.5, -(q as f64), x as f64);
    let r2 = r * r;

    let sin = r * crate::poly(
        r2,
        &[
            3.141_592_653_589_793,
            -5.167_712_780_049_907,
            2.550_164_039_866_279,
            -0.599_264_528_376_357_8,
            0.082_145_841_975_962_36,
            -0.007_369_200_927_300_194,
            0.000_446_633_594_163_232_17,
            0.000_147_229_401_743_877_12,
            -0.000_604_138_393_005_582_7,
        ],
    );
    let cos = crate::poly(
        r2,
        &[
            1.0,
            -4.934_802_200_544_679,
            4.058_712_126_416_669,
            -1.335_262_768_843_029_9,
            0.235_330_629_689_766_45,
            -0.025_806_870_062_591_44,
            0.001_929_193_319_296_624,
            -0.000_101_060_856_947_921_87,
            -9.454_320_539_164_008e-6,
        ],
    );

    match (q as i64) & 3 {
        0 => sin,
        1 => cos,
        2 => -sin,
        _ => -cos,
    }
}

/// Natural logarithm of a positive double-double
///
/// `ln(hi + lo) = ln(hi) + ln(1 + lo/hi) ≈ ln(hi) + lo/hi`, the last term being
/// a tiny correction folded into the double-double `ln`.
#[inline]
fn ln_sum(x: DoubleDouble) -> DoubleDouble {
    crate::f64_::ln_dd(x.high)
        + DoubleDouble {
            high: x.low / x.high,
            low: 0.0,
        }
}

/// Negate a double-double
#[inline]
fn neg(a: DoubleDouble) -> DoubleDouble {
    DoubleDouble {
        high: -a.high,
        low: -a.low,
    }
}

/// Round `value · 2`<sup>`q`</sup> to the nearest `f32`, negating when `negative`
///
/// Covers tgamma's full range: overflow to `±∞` for large `q`, gradual
/// underflow to `±0` for very negative `q`, and [`round_general`] for the
/// normal and subnormal grids in between.
#[inline]
fn finish(value: DoubleDouble, q: i64, negative: bool) -> f32 {
    let magnitude = if q >= 1022 {
        f32::INFINITY
    } else if q <= -203 {
        // `value.high · 2^q` underflows even f64 (minimum f64 ≈ 2^-1074); flush to 0.
        0.0
    } else {
        // Renormalize so `high` is the nearest `f64`: the upstream `DoubleDouble / z` and
        // `DoubleDouble * DoubleDouble` can leave the pair denormal by up to an ulp, which would
        // defeat `round`'s round-to-odd at the hardest f32 boundaries.
        round_general(DoubleDouble::from_sum(
            fast_ldexp(value.high, q),
            fast_ldexp(value.low, q),
        ))
    };

    if negative { -magnitude } else { magnitude }
}

/// `∏_{k=0}^{n-1}(base + k·step)` for `0 ≤ n ≤ 32`, fixed-trip and branch-free
///
/// The recurrence factors `(x−1)(x−2)…` form a product whose length `n = |i|`
/// is uniformly random on random input, so a counted loop's exit, its unroll
/// remainder, and any leftover-factor selects all mispredict — measured ~2+
/// mispredicts per call, the entire gap to CORE-MATH.  Instead, four lanes
/// each run a fixed eight rounds (covering `n ≤ 32`), multiplying by the
/// factor while the lane's index is below `n` and by exact `1.0` after: the
/// compare/blend/multiply packs into ymm ops with no data-dependent branch.
/// Every live factor (`step = ±1`, small integer `k`) is exact, so the only
/// added error is reassociation — a few `2⁻⁵³` ulps, far inside the `2⁻³⁷`
/// gate the [`tgamma`] fast path relies on.
#[inline]
fn recurrence_product(base: f64, step: f64, n: i32) -> f64 {
    let stride = 4.0 * step;
    let nf = f64::from(n);
    let mut p = [1.0_f64; 4];
    let mut f = [
        base,
        base + step,
        crate::fast_mul_add(2.0, step, base),
        crate::fast_mul_add(3.0, step, base),
    ];
    let mut idx = [0.0_f64, 1.0, 2.0, 3.0];

    for _ in 0..8 {
        for j in 0..4 {
            p[j] *= if idx[j] < nf { f[j] } else { 1.0 };
            f[j] += stride;
            idx[j] += 4.0;
        }
    }

    (p[0] * p[2]) * (p[1] * p[3])
}

/// Fast plain-`f64` `Γ(z)` over the recurrence range, with a relative error bound
///
/// Mirrors [`tgamma_dd`] in `f64`: reduce `z` into `[2.375, 3.375]`, evaluate the
/// degree-11 minimax, and walk back by the recurrence.  The factor product (see
/// [`recurrence_product`]) runs on its own dependency chains, overlapping the
/// polynomial, and is folded in with a single multiply or divide — the one
/// remaining data-dependent branch, matching CORE-MATH's.  The error is
/// dominated by the polynomial's `2⁻⁴²`, which the gate in [`tgamma`] uses.
#[inline(always)]
fn tgamma_f64(x: f64) -> (f64, f64) {
    let m = x - TGAMMA_CENTER;
    let i = m.round_ties_even();
    let value = crate::poly(m - i, &TGAMMA_POLY_F64);
    let steps = i.abs() as i32;

    // Γ(x) = Γ(x−i)·∏_{j=1}^{i}(x−j) above the interval, Γ(x−i)/∏_{j=0}^{-i-1}(x+j)
    // below: one parametrization covers both walks (`i = 0` runs an all-dead
    // product, `w = 1`).  Both `x − 0.5` and the `±0.5` add are exact — `x`
    // promotes from `f32` with `|x| > 2⁻¹³`, so the 53-bit span is ample.
    let step = 1.0_f64.copysign(-i);
    let base = crate::fast_mul_add(0.5, step, x - 0.5);
    let w = recurrence_product(base, step, steps);
    let value = if i <= -0.5 { value / w } else { value * w };

    (value, crate::exp2i(-37) * value.abs())
}

/// Double-double `Γ(z)` over the recurrence range, the accurate Ziv fallback
///
/// Reduces `z` into the minimax interval `[2.375, 3.375]` centred on 2.875, then
/// walks back with `Γ(z) = Γ(z−i)·∏(z−j)` for `z` above the interval, or a single
/// reciprocal of `∏(z+j)` below it.  For negative `z` the product runs through
/// negative factors, so it supplies the sign as well — no reflection needed.
#[cold]
#[inline(never)]
fn tgamma_dd(x: f64) -> f32 {
    let m = x - TGAMMA_CENTER;
    let i = m.round_ties_even();
    let mut value = tgamma_poly(m - i);
    let steps = i.abs() as i32;

    if i > 0.0 {
        let mut factor = x;
        for _ in 0..steps {
            factor -= 1.0;
            value = value * factor;
        }
    } else if i < 0.0 {
        let mut product = DoubleDouble { high: x, low: 0.0 };
        let mut factor = x;
        for _ in 1..steps {
            factor += 1.0;
            product = product * factor;
        }
        value = value * product.recip();
    }

    let negative = value.high < 0.0;
    finish(if negative { neg(value) } else { value }, 0, negative)
}

/// The gamma function
#[must_use]
#[inline]
pub fn tgammaf(z: f32) -> f32 {
    if z.is_nan() {
        return z;
    }

    if z == f32::INFINITY {
        return f32::INFINITY;
    }

    if z == 0.0 {
        return f32::INFINITY.copysign(z);
    }

    let x = f64::from(z);

    // Near the pole at 0, Γ(z) ≈ 1/z.  The Maclaurin series Γ(z) = 1/z − γ + c₂z +
    // c₃z² + c₄z³ keeps the dynamic range inside the double-double 1/z while the
    // O(1) correction stays ample in f64; this also yields Γ(±0) = ±∞.
    if z.abs() < 0.000_244_140_63 {
        let correction = crate::poly(
            x,
            &[
                -0.577_215_664_901_532_9,
                0.989_055_995_327_972_6,
                -0.907_479_076_080_886_3,
                0.981_728_086_834_400_2,
            ],
        );
        let value = DoubleDouble::from_quotient(1.0, x)
            + DoubleDouble {
                high: correction,
                low: 0.0,
            };
        return round_signed(value);
    }

    // Γ exceeds f32::MAX at z ≈ 35.0401; bail before the recurrence loop, which
    // would otherwise run unboundedly for huge z.
    if z >= 35.040_1 {
        return f32::INFINITY;
    }

    // Non-positive integers are poles (z = 0 handled above); below −42 the
    // magnitude underflows past 2⁻¹⁵¹ to a signed zero alternating with each pole.
    if x == x.floor() && z < 0.0 {
        return f32::NAN;
    }
    if z < -42.0 {
        return if (x.floor() as i64) & 1 == 0 {
            0.0
        } else {
            -0.0
        };
    }

    // Below −29.625 the recurrence needs more than the 32 steps the fixed-trip
    // product covers; the band is far off any hot path, so the accurate tier
    // serves it directly.
    if z < -29.625 {
        return tgamma_dd(x);
    }

    // Ziv two-step over the recurrence range: the plain-f64 path is correctly
    // rounded unless its value lands within the error bound of an f32 boundary,
    // where the double-double path resolves it.
    let (value, err) = tgamma_f64(x);
    let lo = (value - err) as f32;
    let hi = (value + err) as f32;

    if lo == hi { lo } else { tgamma_dd(x) }
}

/// `ln Γ(y)` as a double-double for `y ≥ ½`
///
/// Reduces `y` upward to `t ≥ 14` with `ln Γ(y) = ln Γ(t) − ln ∏(y+j)`, then
/// applies the Stirling expansion `(t−½)·ln t − t + ½ln(2π) + 1/(12t) +
/// tail(1/t²)`.  The big `(t−½)·ln t − t` and the leading `1/(12t)` stay in
/// double-double; the asymptotic tail is negligible in f64.  Relative error
/// near `2⁻⁶⁴`.
#[inline]
// The Stirling-reduction loop increments `t` by exactly 1.0 each pass, which is
// exact in f64, so the float-condition `while t < LGAMMA_STIRLING` terminates cleanly.
#[allow(clippy::while_float)]
fn lgamma_pos_dd(y: f64) -> DoubleDouble {
    let mut product = DoubleDouble {
        high: 1.0,
        low: 0.0,
    };
    let mut t = y;
    let mut reduced = false;

    while t < LGAMMA_STIRLING {
        product = product * t;
        t += 1.0;
        reduced = true;
    }

    let u = 1.0 / (t * t);
    let rest = crate::poly(u, &LGAMMA_TAIL) * (u / t);

    let stirling = crate::f64_::ln_dd(t) * (t - 0.5)
        + DoubleDouble { high: -t, low: 0.0 }
        + HALF_LN_2PI
        + DoubleDouble::from_quotient(1.0, 12.0 * t)
        + DoubleDouble {
            high: rest,
            low: 0.0,
        };

    if reduced {
        stirling + neg(ln_sum(product))
    } else {
        stirling
    }
}

/// `ln Γ(y)` in plain `f64` for `y ≥ ½`, the fast Ziv path
///
/// Below 8 a log-free rational `(y−1)(y−2)·g(y)` does the work; from 8 up the
/// Stirling series with a single logarithm takes over (its `2⁻⁴⁶` tail
/// truncation sits well inside the [`lgamma`] gate).
#[inline]
fn lgamma_pos_f64(y: f64) -> f64 {
    if y < 8.0 {
        let g = crate::poly(y, &LGAMMA_NUM) / crate::poly(y, &LGAMMA_DEN);
        return (y - 1.0) * (y - 2.0) * g;
    }

    // One division serves all three reciprocal factors: `1/(12y) + P(u)·u/y
    // = v·(1/12 + P(u)·u)` with `v = 1/y`, `u = v²`.  The extra roundings
    // land on `s` (≤ 1/96 of the result) — invisible next to the `2⁻³⁶` gate.
    // `log_lookup` replaces the libm `ln` for the same reason lgamma's f64
    // sibling owns its logs: shorter latency, same ≈2⁻⁵² accuracy class.
    let v = 1.0 / y;
    let u = v * v;
    let s = v * crate::fast_mul_add(crate::poly(u, &LGAMMA_TAIL), u, 1.0 / 12.0);
    let ln = log_lookup(y, &LNF_TABLES);
    crate::fast_mul_add(y - 0.5, ln, -y) + 0.918_938_533_204_672_8 + s
}

/// Fast `f64` approximation of `ln|Γ(z)|` with an absolute error bound
///
/// The fast path's relative error (`≈2⁻³⁷` from the rational, less from Stirling)
/// turns into an *absolute* error proportional to the magnitude actually fed
/// through it — `ln Γ(1−z)` under reflection, where the reflected result itself
/// can be tiny.  Returning the bound lets [`lgamma`] gate tightly instead of
/// assuming the worst case everywhere.
#[inline]
fn lgamma_f64(z: f32) -> (f64, f64) {
    /// `ln(π)`, the reflection constant (kept literal — `f64::ln` is not `const`)
    const LN_PI: f64 = 1.144_729_885_849_400_2;

    let x = f64::from(z);
    if z < 0.5 {
        // `log_lookup` for the sine's logarithm: ≈2⁻⁵² relative on
        // `|ln sin πz| ≤ ~15` stays under the 2⁻⁴⁴ absolute floor below.
        let reflected = lgamma_pos_f64(1.0 - x);
        let value = LN_PI - log_lookup(abs_sinpi(z), &LNF_TABLES) - reflected;
        (
            value,
            crate::exp2i(-36) * reflected.abs() + crate::exp2i(-44),
        )
    } else {
        let value = lgamma_pos_f64(x);
        (value, crate::exp2i(-36) * value.abs() + crate::exp2i(-44))
    }
}

/// The double-double `ln|Γ(z)|`, the accurate Ziv fallback for [`lgamma`]
///
/// The caller has already handled NaN, ±∞, the non-positive-integer poles and
/// the exact zeros `Γ(1) = Γ(2) = 1`.  Kept out of line so the rare fallback
/// never bloats the hot path.
#[cold]
#[inline(never)]
fn lgamma_dd(z: f32) -> f32 {
    if z < 0.5 {
        // ln|Γ(z)| = ln π − ln|sin(πz)| − ln Γ(1−z), the reflection formula.
        let value = ln_sum(PI)
            + neg(crate::f64_::ln_dd(sinpi(z).abs()))
            + neg(lgamma_pos_dd(1.0 - f64::from(z)));
        return round_signed(value);
    }

    round_signed(lgamma_pos_dd(f64::from(z)))
}

/// The natural logarithm of the absolute value of the gamma function
#[must_use]
#[inline]
pub fn lgammaf(z: f32) -> f32 {
    if z == 0.0 || z == f32::INFINITY {
        return f32::INFINITY;
    }

    if z.is_nan() {
        return z;
    }

    // Non-positive integers (and −∞) are poles; Γ(1) = Γ(2) = 1 give exact zeros.
    if z < 0.5 && z.round_ties_even() == z {
        return f32::INFINITY;
    }
    if z == 1.0 || z == 2.0 {
        return 0.0;
    }

    // Ziv two-step: the plain-f64 path is correctly rounded unless the value
    // lands within `err` of an f32 boundary, where the double-double path
    // resolves it.
    let (f, err) = lgamma_f64(z);
    let lo = (f - err) as f32;
    let hi = (f + err) as f32;

    if lo == hi { lo } else { lgamma_dd(z) }
}
