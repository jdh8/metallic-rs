use super::{EXP_SHIFT, LN_2_HI, LN_2_LO, Magnitude, normalize};
use crate::f64::double::fast_ldexp;

/// Polynomial approximation of restriction of `(exp(x) - 1) / x`
/// to `-0.5 * ln(2) ..= 0.5 * ln(2)`
///
/// In geometry, this function returns the slope of the secant line between the
/// points `(0, 1)` and `(x, exp(x))` on the graph of the exponential function.
#[inline]
fn exp_slope(x: f64) -> f64 {
    crate::poly(
        x,
        &[
            1.0,
            5.000_000_000_000_006e-1,
            1.666_666_666_666_660_7e-1,
            4.166_666_666_657_388_4e-2,
            8.333_333_333_377_178e-3,
            1.388_888_893_226_473e-3,
            1.984_126_974_692_237_6e-4,
            2.480_150_459_649_619_4e-5,
            2.755_738_188_469_386e-6,
            2.762_626_468_193_145e-7,
            2.506_206_487_727_576e-8,
        ],
    )
}

/// `32 · log₂e = 32 / ln2`, the scale of the `exp_m1` table reduction
const FRAC_32_LN_2: f64 = 46.166_241_308_446_83;

/// `2^(j/32)` for `j` in `0..32` — the table for [`exp_m1`]'s `1/32`-step reduction
///
/// Built from `mpmath.power(2, j/32)` rounded to `f64`.
const EXP2_32: [f64; 32] = [
    1.0,
    1.021_897_148_654_116_6,
    1.044_273_782_427_413_8,
    1.067_140_400_676_823_7,
    1.090_507_732_665_257_7,
    1.114_386_742_595_892_4,
    1.138_788_634_756_691_6,
    1.163_724_858_777_577_5,
    1.189_207_115_002_721,
    1.215_247_359_980_469,
    1.241_857_812_073_484,
    1.269_050_957_191_733_2,
    1.296_839_554_651_009_6,
    1.325_236_643_159_741_3,
    1.354_255_546_936_892_7,
    1.383_909_881_963_832,
    1.414_213_562_373_095_1,
    1.445_180_806_977_046_7,
    1.476_826_145_939_499_3,
    1.509_164_427_593_422_8,
    1.542_210_825_407_940_7,
    1.575_980_845_107_886_5,
    1.610_490_331_949_254_3,
    1.645_755_478_153_965,
    1.681_792_830_507_429,
    1.718_619_298_122_478,
    1.756_252_160_373_299_5,
    1.794_709_075_003_107_2,
    1.834_008_086_409_342_4,
    1.874_167_634_110_3,
    1.915_206_561_397_147_4,
    1.957_144_124_175_400_2,
];

/// Minimax of `2^(h/32)` on `h ∈ [−½, ½]`, low-degree first — `exp_m1`'s kernel
///
/// Degree 4, max absolute error `2⁻⁴³·⁵`, from
/// `mpmath.chebyfit(lambda h: 2**(h/32), [-0.5, 0.5], 5)`.  Multiplied by a table
/// entry `2^(j/32)` and scaled by `2^q`, it reconstructs `exp(x)` over a `1/32`-
/// step grid, so the polynomial only spans a `1/64`-wide band of `ln2`.
const EXP2_32_POLY: [f64; 5] = [
    1.0,
    0.021_660_849_391_722_17,
    0.000_234_596_198_199_444_9,
    1.693_863_390_315_564_4e-6,
    9.172_607_532_092_245e-9,
];

/// Finite `exp` for correctly-rounded `f32`
#[inline]
pub(super) fn finite_exp(x: f64) -> f64 {
    let n = (x * core::f64::consts::LOG2_E).round_ties_even();
    let x = crate::fast_mul_add(n, -LN_2_HI, x);
    let x = crate::fast_mul_add(n, -LN_2_LO, x);
    let y = crate::fast_mul_add(exp_slope(x), x, 1.0);

    fast_ldexp(y, n as i64)
}

/// The exponential function
#[must_use]
#[inline]
pub fn exp(x: f32) -> f32 {
    use core::f32::consts::LN_2;

    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 * LN_2 {
        return 0.0;
    }

    if x > f32::MAX_EXP as f32 * LN_2 {
        return f32::INFINITY;
    }

    finite_exp(x.into()) as f32
}

/// Raise 2 to the power of `x`
#[must_use]
#[inline]
pub fn exp2(x: f32) -> f32 {
    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 {
        return 0.0;
    }

    if x > f32::MAX_EXP as f32 {
        return f32::INFINITY;
    }

    if x.eq(&-0.029_743_774) {
        return 0.979_594_3;
    }

    let n = x.round_ties_even();
    let x = crate::poly(
        (x - n).into(),
        &[
            1.0,
            6.931_471_805_599_497e-1,
            2.402_265_069_590_928e-1,
            5.550_410_866_446_075e-2,
            9.618_129_107_922_14e-3,
            1.333_355_822_832_622_6e-3,
            1.540_353_006_215_995_4e-4,
            1.525_265_822_994_141_7e-5,
            1.321_562_299_026_546e-6,
            1.020_851_913_096_976e-7,
            7.043_007_099_737_507e-9,
        ],
    );

    fast_ldexp(x, n as i64) as f32
}

/// Raise 10 to the power of `x`
#[must_use]
#[inline]
pub fn exp10(x: f32) -> f32 {
    use core::f32::consts::LOG10_2;
    const LOG10_2_HI: f64 = 0.301_029_995_664_066_5;
    const LOG10_2_LO: f64 = -8.532_344_317_057_107e-14;

    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 * LOG10_2 {
        return 0.0;
    }

    if x > f32::MAX_EXP as f32 * LOG10_2 {
        return f32::INFINITY;
    }

    let x: f64 = x.into();
    let n = (x * core::f64::consts::LOG2_10).round_ties_even();
    let x = crate::fast_mul_add(n, -LOG10_2_HI, x);
    let x = crate::fast_mul_add(n, -LOG10_2_LO, x);
    let x = crate::poly(
        x,
        &[
            1.0,
            2.302_585_092_994_048_6,
            2.650_949_055_239_204_5,
            2.034_678_592_287_247,
            1.171_255_148_908_203,
            5.393_829_313_950_126e-1,
            2.069_958_495_746_965_8e-1,
            6.808_909_329_404_776e-2,
            1.959_761_565_686_179e-2,
            5.027_633_471_110_143e-3,
            1.157_655_379_074_781_8e-3,
        ],
    );

    fast_ldexp(x, n as i64) as f32
}

/// Compute `exp(x) - 1` accurately especially for small `x`
///
/// With `a = x·32·log₂e`, `m = round(a) = 32q + j`, and `h = a − m ∈ [−½, ½]`,
/// the result is `2^q·2^(j/32)·2^(h/32) − 1`.  The `1/32`-step table folds `ln2`
/// into the reduction, so the fast path costs one multiply, a round, and a
/// subtract (no two-word `ln2` split) plus a degree-4 kernel; one exponent
/// injection (`fast_ldexp` on the table entry) supplies `2^q·2^(j/32)`.
///
/// The naive `2^(m/32)·2^(h/32) − 1` cancels for the small results near `m = 0`,
/// where the fractional table cannot form an exact `2^(m/32) − 1`.  A Ziv gate at
/// `2⁻⁴²` (the fast path is good to ≈2⁻⁴³ relative) hands those few ambiguous
/// inputs to the two-word, degree-10 path, which splits off the exact `2ⁿ − 1`.
///
/// - `m = 0` (`|x| ≤ ln2/64`): return `x·exp_slope(x) = exp(x) − 1` directly,
///   so the cancellation a literal `exp(x) − 1` would suffer never arises.
#[must_use]
#[inline]
pub fn exp_m1(x: f32) -> f32 {
    use core::f32::consts::LN_2;

    if x < (f32::MANTISSA_DIGITS + 1) as f32 * -LN_2 {
        return -1.0;
    }

    if x > f32::MAX_EXP as f32 * LN_2 {
        return f32::INFINITY;
    }

    /// `1.5 · 2⁵²`: adding it rounds a small `f64` to the nearest integer
    /// (ties to even) and parks that integer in the low mantissa bits, biased by
    /// `2⁵¹`, so `m = round(a)` and its bit pattern come out together.
    const BIG: f64 = f64::from_bits(0x4338_0000_0000_0000);

    let x: f64 = x.into();
    let a = x * FRAC_32_LN_2;
    let abig = a + BIG;
    let m = abig - BIG;

    if m == 0.0 {
        return (x * exp_slope(x)) as f32;
    }

    // Low 52 bits of `abig` hold `2⁵¹ + m`; `j = m & 31` (since `2⁵¹ ≡ 0 mod 32`)
    // indexes the table and `q = m >> 5` scales it — no `f64 → int` conversion.
    let u = abig.to_bits();
    let q = (((u & 0x000F_FFFF_FFFF_FFFF) as i64) - 0x0008_0000_0000_0000) >> 5;
    let h = a - m;
    let sv = fast_ldexp(EXP2_32[(u & 31) as usize], q);
    let r = crate::fast_mul_add(crate::poly(h, &EXP2_32_POLY), sv, -1.0);

    // Ziv gate: the fast path is good to ≈2⁻⁴³ relative to `exp(x) ≈ sv`, so if
    // both ends of the `±sv·2⁻⁴²` error interval round to the same `f32`, that
    // `f32` is correct; otherwise refine.  Scaling the gate by `sv` (not by `r`)
    // keeps it valid through the `m = ±1` cancellation, where `r` is tiny.
    let epsilon = sv * crate::exp2i(-42);
    let lower = (r - epsilon) as f32;

    if lower == (r + epsilon) as f32 {
        return lower;
    }

    // Accurate fallback: two-word `ln2` reduction, splitting off the exact
    // `2ⁿ − 1` so the small-result cancellation never bites.
    let n = (x * core::f64::consts::LOG2_E).round_ties_even();
    let r = crate::fast_mul_add(n, -LN_2_HI, x);
    let r = crate::fast_mul_add(n, -LN_2_LO, r);
    let y = exp_slope(r);

    (fast_ldexp(r * y, n as i64) + (crate::exp2i(n as i64) - 1.0)) as f32
}

/// Multiply `x` by 2 raised to the power of `n`
#[must_use]
#[inline]
pub const fn ldexp(x: f32, n: i32) -> f32 {
    const MIN_EXP: i32 = f64::MIN_EXP - 1;
    const MAX_EXP: i32 = f64::MAX_EXP;

    let coefficient = match n {
        ..MIN_EXP => 0.5 * f64::MIN_POSITIVE,
        n @ MIN_EXP..MAX_EXP => f64::from_bits(((MAX_EXP - 1 + n) as u64) << crate::f64::EXP_SHIFT),
        MAX_EXP.. => f64::MAX,
    };

    (x as f64 * coefficient) as f32
}

/// Decompose into a significand and an exponent
///
/// The absolute value of the significand is in the range of [0.5, 1) for
/// nonzero finite `x` for historical reasons.  This function also explains how
/// [`f32::MAX_EXP`] and [`f32::MIN_EXP`] are defined.
#[must_use]
#[inline]
pub const fn frexp(x: f32) -> (f32, i32) {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return (x, 0);
    };

    let mask = f32::MIN_POSITIVE.to_bits() - 1;
    let significand = magnitude as u32 & mask | 0.5f32.to_bits();

    (
        f32::from_bits(crate::u32_sign_bit(sign) | significand),
        f32::MIN_EXP - 1 + (magnitude >> EXP_SHIFT),
    )
}
