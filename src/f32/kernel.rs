use crate::f64::kernel::Sum;

/// Fast C `ldexp` assuming normal argument and result
#[inline]
pub const fn fast_ldexp(x: f64, n: i64) -> f64 {
    const SHIFT: u32 = f64::MANTISSA_DIGITS - 1;
    f64::from_bits((x.to_bits() as i64 + (n << SHIFT)) as u64)
}

/// Polynomial approximation of restriction of `(exp(x) - 1) / x`
/// to `-0.5 * ln(2) ..= 0.5 * ln(2)`
///
/// In geometry, this function returns the slope of the secant line between the
/// points `(0, 1)` and `(x, exp(x))` on the graph of the exponential function.
#[inline]
pub fn exp_slope(x: f64) -> f64 {
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

/// Polynomial approximation of inverse hyperbolic tangent restricted to
/// `-c..=c`, where
///
/// ```text
///     √2 - 1                  1 + c
/// c = ------  the solution to ----- = √2.
///     √2 + 1,                 1 - c
/// ```
#[inline]
pub fn atanh(x: f64) -> f64 {
    let y = x * x;
    let y = y * crate::poly(
        y,
        &[
            0.333_333_333_333_310_1,
            0.200_000_000_056_551_2,
            0.142_857_120_550_553_72,
            0.111_114_324_826_276_95,
            0.090_700_447_553_529_28,
            0.083_116_173_891_988_07,
        ],
    );
    crate::mul_add(y, x, x)
}

/// Base 2 logarithm for a finite positive `f64`
#[inline]
pub fn log2(x: f64) -> f64 {
    use crate::f64::EXP_SHIFT;
    use core::f64::consts;

    #[allow(clippy::cast_possible_wrap)]
    let i = x.to_bits() as i64;

    #[allow(clippy::cast_possible_wrap)]
    let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;

    #[allow(clippy::cast_sign_loss)]
    let x = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);

    #[allow(clippy::cast_precision_loss)]
    crate::mul_add(
        2.0 * consts::LOG2_E,
        atanh((x - 1.0) / (x + 1.0)),
        exponent as f64,
    )
}

/// [`f64::exp2`] with precision of `f32`
///
/// This function is especially useful for computing `x`<sup>`y`</sup> when
/// combined with [`log2`].
#[inline]
pub fn exp2(x: f64) -> f64 {
    if x < (f64::MIN_EXP - 1).into() {
        return 0.0;
    }

    if x > f64::MAX_EXP.into() {
        return f64::INFINITY;
    }

    let n = x.round_ties_even();
    let x = crate::poly(
        x - n,
        &[
            1.0,
            6.931_471_880_289_533e-1,
            2.402_265_108_421_173_5e-1,
            5.550_357_105_498_874_4e-2,
            9.618_030_771_171_498e-3,
            1.339_086_685_300_951e-3,
            1.546_973_499_989_028_8e-4,
        ],
    );

    // SAFETY: `n` is within `(f64::MIN_EXP - 1) ..= f64::MAX_EXP`
    fast_ldexp(x, unsafe { n.to_int_unchecked() })
}

/// Argument reduction for trigonometric functions
///
/// - `x`: finite radians with a positive sign bit
///
/// The prototype of this function resembles `__rem_pio2` in GCC, but this
/// function is only for `f32`.  Pseudocode is as follows.
///
/// ```text
/// quotient = nearest integer of x / (π/2)
/// y = x - quotient * (π/2) // IEEE remainder of x / (π/2)
/// (quotient, y)
/// ```
///
/// The lowest 2 bits of the returned quotient are accurate.
#[inline]
pub fn rem_pio2(x: f32) -> (i64, f64) {
    use core::f64::consts;
    debug_assert!(x.is_sign_positive());

    /// π/2 with the highest [`f32::MANTISSA_DIGITS`] (24) bits
    const PI_2_HI: f64 = 1.570_796_310_901_641_8;

    /// Bits of π/2 below [`PI_2_HI`]
    const PI_2_LO: f64 = 1.589_325_477_352_819_6e-8;

    /// Little-endian 256 bits of 2/π
    const FRAC_2_PI: [u64; 4] = [
        0xFE51_63AB_DEBB_C561,
        0xDB62_9599_3C43_9041,
        0xFC27_57D1_F534_DDC0,
        0xA2F9_836E_4E44_1529,
    ];

    if x < core::f32::consts::PI * crate::exp2i(27) as f32 {
        let x: f64 = x.into();
        let q = (x * consts::FRAC_2_PI).round_ties_even();
        let y = crate::mul_add(q, -PI_2_HI, x);
        let y = crate::mul_add(q, -PI_2_LO, y);

        // SAFETY: q < 2^28
        return (unsafe { q.to_int_unchecked() }, y);
    }

    let magnitude = x.to_bits();
    let significand: u128 = ((magnitude & 0x007F_FFFF) | 0x0080_0000).into();
    let p0 = significand * u128::from(FRAC_2_PI[0]);
    let p1 = significand * u128::from(FRAC_2_PI[1]) + (p0 >> 64);
    let p2 = significand * u128::from(FRAC_2_PI[2]) + (p1 >> 64);
    let high = significand * u128::from(FRAC_2_PI[3]) + (p2 >> 64);
    let low = p2 << 64 | p1 << 64 >> 64;
    let shift = (magnitude >> super::EXP_SHIFT) - 150;
    let product = high << shift | low >> (128 - shift);
    let r = product as i64;
    let q = (product >> 64) as i64;

    (
        q.wrapping_sub(r >> 63),
        consts::PI * crate::exp2i(-65) * r as f64,
    )
}

/// Cosine restricted to `-π/4..=π/4`
#[inline]
pub fn cos(x: f64) -> f32 {
    crate::poly(
        x * x,
        &[
            1.0,
            -4.999_999_999_999_946_7e-1,
            4.166_666_666_650_087e-2,
            -1.388_888_887_158_942_7e-3,
            2.480_157_897_844_104e-5,
            -2.755_529_138_739_507_4e-7,
            2.063_333_980_512_758_6e-9,
        ],
    ) as f32
}

/// Sine restricted to `-π/4..=π/4`
#[inline]
pub fn sin(x: f64) -> f32 {
    let y = x * x;
    let y = y * crate::poly(
        y,
        &[
            -1.666_666_666_666_663e-1,
            8.333_333_333_321_917e-3,
            -1.984_126_982_945_719_3e-4,
            2.755_731_358_196_805e-6,
            -2.505_074_230_488_205e-8,
            1.589_594_452_434_234_8e-10,
        ],
    );
    crate::mul_add(y, x, x) as f32
}

/// `|sin(πx)|`, accurate to ≈2⁻⁵¹ relative — the lgamma reflection's sine factor
///
/// Reduces to the distance to the nearest integer, `r = x − round(x) ∈ [−½, ½]`,
/// where `|sin(πx)| = |r|·P(r²)`.  A single odd polynomial — no quadrant branch
/// and no second cosine series like [`sinpi`] — keeps the `≈π|r|` behaviour exact
/// at the integer poles, where `ln|sin(πx)|` and hence lgamma blow up.  The
/// minimax `P(u) ≈ sin(π√u)/√u` on `u ∈ [0, ¼]` is from
/// `mpmath.chebyfit(lambda u: sin(pi·√u)/√u, [0, 0.25], 8)`.
#[inline]
pub fn abs_sinpi(x: f32) -> f64 {
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
pub fn sinpi(x: f32) -> f64 {
    let x = 2.0f32.mul_add(-(0.5 * x).round_ties_even(), x);
    let q = (2.0 * x).round_ties_even();
    let r = f64::from(0.5f32.mul_add(-q, x));
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

/// `atan(k / 8)` as a double-double for `k` in `0..=8`
const ATAN_TABLE: [(f64, f64); 9] = [
    (0.0, 0.0),
    (0.124_354_994_546_761_44, -3.125_324_142_453_938_3e-18),
    (0.244_978_663_126_864_14, 1.069_875_561_873_445_1e-17),
    (0.358_770_670_270_572_25, -2.462_381_558_263_863_5e-17),
    (0.463_647_609_000_806_1, 2.269_877_745_296_168_7e-17),
    (0.558_599_315_343_562_4, -5.455_630_548_591_626_4e-18),
    (0.643_501_108_793_284_4, 1.583_478_505_144_428_6e-17),
    (0.718_829_999_621_624_5, -2.147_838_844_445_698_3e-17),
    (0.785_398_163_397_448_3, 3.061_616_997_868_383e-17),
];

/// π/2 as a double-double
const FRAC_PI_2: Sum = Sum {
    high: 1.570_796_326_794_896_6,
    low: 6.123_233_995_736_766e-17,
};

/// π as a double-double
pub const PI: Sum = Sum {
    high: 3.141_592_653_589_793,
    low: 1.224_646_799_147_353_2e-16,
};

/// Arctangent of a double-double in `[0, 1]`, returned as a double-double
///
/// The argument is reduced into a cell of width 1/8 centred on `c = k/8`, where
/// the table holds `atan(c)`.  Inside the cell `atan(q) = atan(c) + atan(u)`
/// with `u = (q - c) / (1 + q·c)` and `|u| ≤ 1/16`, so the Taylor series of the
/// odd part `atan(u) - u` converges in a handful of terms evaluated in `f64`.
#[inline]
fn atan_dd(q: Sum) -> Sum {
    let k = (q.high * 8.0).round_ties_even();
    let c = k * 0.125;

    // `q.high ∈ [0, 1]` ⇒ `k ∈ {0, …, 8}`, always a valid index
    let (high, low) = ATAN_TABLE[k as usize];

    // u = (q - c) / (1 + q·c), exact in double-double (c is a power-of-two multiple)
    let u = (q + Sum { high: -c, low: 0.0 })
        * (q * c
            + Sum {
                high: 1.0,
                low: 0.0,
            })
        .recip();

    // atan(u) = u - u³/3 + u⁵/5 - u⁷/7 + …, |u| ≤ 1/16.  The cubic term carries
    // the result near small `q`, so it is formed in double-double (`/3.0` is the
    // compensated divisor); the remaining tail is tiny and stays in `f64`.
    let uu = u.high * u.high;
    let cubic = u * u * u / 3.0;
    let tail = u.high
        * uu
        * uu
        * crate::poly(
            uu,
            &[
                1.0 / 5.0,
                -1.0 / 7.0,
                1.0 / 9.0,
                -1.0 / 11.0,
                1.0 / 13.0,
                -1.0 / 15.0,
                1.0 / 17.0,
                -1.0 / 19.0,
                1.0 / 21.0,
            ],
        );

    Sum { high, low }
        + u
        + Sum {
            high: -cubic.high,
            low: -cubic.low,
        }
        + Sum {
            high: tail,
            low: 0.0,
        }
}

/// Round a normalized positive double-double to the nearest `f32`
///
/// A plain `value.high as f32` can double-round when `value.high` lands on an
/// `f32` midpoint: the cast rounds to even before the low word breaks the tie.
/// Rounding `value.high` to odd in `f64` first (in the direction of `value.low`)
/// sidesteps this — every `f32` midpoint has at least 28 trailing zero bits in
/// `f64`, hence is even, so the odd nudge lands on the correct side before the
/// final round to nearest.
#[inline]
pub fn round(value: Sum) -> f32 {
    let bits = value.high.to_bits();

    let odd = if value.low == 0.0 || bits & 1 == 1 {
        value.high
    } else if value.low > 0.0 {
        f64::from_bits(bits + 1)
    } else {
        f64::from_bits(bits - 1)
    };

    odd as f32
}

/// Round a non-negative double-double to the nearest `f32`, safe across the
/// subnormal range and overflow
///
/// In the normal range this is [`round`] (round-to-odd then cast, which sends
/// overflow to `+∞`).  Below `f32::MIN_POSITIVE` a plain cast would round twice,
/// so the value is quantized once on the `2⁻¹⁴⁹` subnormal grid, exactly like
/// the subnormal branch of the `f64` exponential.
#[inline]
pub fn round_general(value: Sum) -> f32 {
    if value.high >= f64::from(f32::MIN_POSITIVE) {
        return round(value);
    }

    let high = value.high * crate::exp2i(149);
    let low = value.low * crate::exp2i(149);
    let n = high.round_ties_even();
    let n = n + ((high - n) + low).round_ties_even();

    (n * crate::exp2i(-149)) as f32
}

/// Round a signed normal-range double-double to the nearest `f32`
///
/// Rounds the magnitude to odd then restores the sign, so [`round`]'s
/// positive-only round-to-odd applies on either side of zero.
#[inline]
pub fn round_signed(value: Sum) -> f32 {
    let magnitude = round(Sum {
        high: value.high.abs(),
        low: if value.high < 0.0 {
            -value.low
        } else {
            value.low
        },
    });

    magnitude.copysign(value.high as f32)
}

/// Center of the [`tgamma_poly`] minimax interval, `Γ(TGAMMA_CENTER + d)`
pub const TGAMMA_CENTER: f64 = 2.875;

/// Leading double-double coefficients of `Γ(2.875 + d)`, `c₀..c₅`
///
/// Paired with [`TGAMMA_TAIL`]; together a degree-18 minimax fit on `d ∈ [−½, ½]`
/// with relative approximation error `2⁻⁶⁷`, from
/// `mpmath.chebyfit(lambda d: gamma(mpf("2.875")+d), [-0.5,0.5], 19)` at
/// `prec=260`.  Only the leading six coefficients need double-double — `c_k`'s
/// f64 rounding contributes `2⁻⁵³·|c_k|·½ᵏ`, which drops below `2⁻⁶²` from `c₆`.
const TGAMMA_DD: [Sum; 6] = [
    Sum {
        high: 1.7877108988969403,
        low: -3.737560105011311e-17,
    },
    Sum {
        high: 1.5591939012079505,
        low: -6.845438926265673e-18,
    },
    Sum {
        high: 1.051049326681183,
        low: -8.10863306869118e-17,
    },
    Sum {
        high: 0.47065801829339715,
        low: 1.1343489177185007e-18,
    },
    Sum {
        high: 0.18881863832011508,
        low: -3.2013513690009033e-18,
    },
    Sum {
        high: 0.05883154841060908,
        low: -4.38554794255165e-20,
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
pub fn tgamma_poly(d: f64) -> Sum {
    let tail = crate::poly(d, &TGAMMA_TAIL);

    let mut acc = TGAMMA_DD[5] + Sum::from_product(d, tail);
    for coefficient in TGAMMA_DD[..5].iter().rev() {
        acc = acc * d + *coefficient;
    }

    acc
}

/// `Γ(2.875 + d)` for `d ∈ [−½, ½]` as a plain-`f64` degree-11 minimax
///
/// Relative error `2⁻⁴²` — the fast Ziv path of [`tgamma`] only needs enough to
/// gate against the double-double [`tgamma_poly`], so half the degree suffices.
pub const TGAMMA_POLY_F64: [f64; 12] = [
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
pub const HALF_LN_2PI: Sum = Sum {
    high: 0.9189385332046728,
    low: -3.878_294_158_067_241_4e-17,
};

/// Argument above which the Stirling series for `ln Γ` converges fast enough
pub const LGAMMA_STIRLING: f64 = 14.0;

/// Stirling tail `Σ_{k≥2} B₂ₖ/(2k(2k−1)) t^{1−2k}` past the leading `1/(12t)`
///
/// Coefficients `B₄/12`, `B₆/30`, … the Bernoulli factors of `ln Γ`'s asymptotic
/// expansion; evaluated as `(u/t)·poly(u)` with `u = 1/t²`.
pub const LGAMMA_TAIL: [f64; 7] = [
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
pub const LGAMMA_NUM: [f64; 8] = [
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
pub const LGAMMA_DEN: [f64; 8] = [
    0.002_584_416_514_557_812,
    0.090_842_150_680_063_21,
    0.556_476_452_487_686_2,
    1.0,
    0.628_146_845_505_252_6,
    0.142_068_778_652_565_23,
    0.010_505_967_556_747_394,
    0.000_179_700_933_585_125_6,
];

/// Magnitude of `atan2(y, x)` for finite nonzero `a = |x|`, `b = |y|`
///
/// Returns the correctly-rounded angle in `[0, π]`; the caller restores the
/// sign of `y` with `copysign`.
#[inline]
pub fn atan2(a: f64, b: f64, x_negative: bool) -> f32 {
    let phi = if a >= b {
        atan_dd(Sum::from_quotient(b, a))
    } else {
        // π/2 - atan(a/b)
        let t = atan_dd(Sum::from_quotient(a, b));
        FRAC_PI_2
            + Sum {
                high: -t.high,
                low: -t.low,
            }
    };

    let theta = if x_negative {
        // π - φ
        PI + Sum {
            high: -phi.high,
            low: -phi.low,
        }
    } else {
        phi
    };

    round(theta)
}
