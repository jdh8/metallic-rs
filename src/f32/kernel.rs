use core::num::Wrapping;

/// Fast C `ldexp` assuming normal argument and result
#[inline]
pub fn fast_ldexp(x: f64, n: i64) -> f64 {
    const SHIFT: u32 = f64::MANTISSA_DIGITS - 1;

    #[allow(clippy::cast_possible_wrap)]
    let wrapped = x.to_bits() as i64;

    #[allow(clippy::cast_sign_loss)]
    return f64::from_bits((wrapped + (n << SHIFT)) as u64);
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

    #[allow(clippy::cast_possible_truncation)]
    return fast_ldexp(x, n as i64);
}

/// Argument reduction for trigonometric functions
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
/// The returned quotient has accurate sign and last 2 bits.
#[inline]
pub fn rem_pio2(x: f32) -> (i32, f64) {
    /// Get 96 bits of 2/π with `offset` bits skipped
    #[inline]
    const fn segment(offset: usize) -> (Wrapping<u64>, Wrapping<u32>) {
        const FRAC_2_PI: [u64; 4] = [
            0xA2F9_836E_4E44_1529,
            0xFC27_57D1_F534_DDC0,
            0xDB62_9599_3C43_9041,
            0xFE51_63AB_DEBB_C561,
        ];

        let (index, shift) = (offset / 64, offset % 64);

        let high = if shift == 0 {
            FRAC_2_PI[index]
        } else {
            FRAC_2_PI[index] << shift | FRAC_2_PI[index + 1] >> (64 - shift)
        };

        let low = if shift > 32 {
            FRAC_2_PI[index + 1] << (shift - 32) | FRAC_2_PI[index + 2] >> (96 - shift)
        } else {
            FRAC_2_PI[index + 1] >> (32 - shift)
        };

        (Wrapping(high), Wrapping(low as u32))
    }

    /// π/2 with the highest [`f32::MANTISSA_DIGITS`] (24) bits
    const PI_2_HI: f64 = 1.570_796_310_901_641_8;

    /// Bits of π/2 below [`PI_2_HI`]
    const PI_2_LO: f64 = 1.589_325_477_352_819_6e-8;

    /// π * 2^-65
    const PI_2_65: f64 = 8.515_303_950_216_386e-20;
    const _2_32: f64 = (1u64 << 32) as f64;
    const _: () = assert!(_2_32 * _2_32 * PI_2_65 == core::f64::consts::FRAC_PI_2);

    let magnitude = x.abs().to_bits();

    // Non-finite
    if magnitude >= 0x7F80_0000 {
        return (0, f64::NAN);
    }

    // |x| < π * 2^27
    if magnitude < 0x4DC9_0FDB {
        let x: f64 = x.into();
        let q = (core::f64::consts::FRAC_2_PI * x).round_ties_even() + 0.0;
        let y = crate::mul_add(q, -PI_2_HI, x);
        let y = crate::mul_add(q, -PI_2_LO, y);
        return (q as i32, y);
    }

    let (high, low) = segment((magnitude >> super::EXP_SHIFT) as usize - 152);
    let low: Wrapping<u64> = Wrapping(low.0.into());
    let significand: Wrapping<u64> = Wrapping(((magnitude & 0x007F_FFFF) | 0x0080_0000).into());

    // First 64 bits of fractional part of x/(2π)
    let product = significand * high + ((significand * low) >> 32);
    let r = (product.0 << 2) as i64;
    let q = (product.0 >> 62) as i32 + i32::from(r < 0);
    let q = if x.is_sign_negative() { -q } else { q };
    let y = PI_2_65.copysign(x.into()) * (r as f64);

    (q, y)
}

/// Cosine restricted to `-π/4..=π/4`
#[inline]
pub fn cos(x: f64) -> f64 {
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
    )
}

/// Sine restricted to `-π/4..=π/4`
#[inline]
pub fn sin(x: f64) -> f64 {
    let xx = x * x;
    let y = crate::poly(
        xx,
        &[
            -1.666_666_666_666_663e-1,
            8.333_333_333_321_917e-3,
            -1.984_126_982_945_719_3e-4,
            2.755_731_358_196_805e-6,
            -2.505_074_230_488_205e-8,
            1.589_594_452_434_234_8e-10,
        ],
    );
    crate::mul_add(crate::mul_add(y, xx, 0.0), x, x)
}
