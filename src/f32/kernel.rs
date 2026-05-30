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
const PI: Sum = Sum {
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
