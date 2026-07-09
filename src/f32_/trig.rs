use super::EXP_SHIFT;

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
fn rem_pio2(x: f32) -> (i64, f64) {
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
        let y = crate::fast_mul_add(q, -PI_2_HI, x);
        let y = crate::fast_mul_add(q, -PI_2_LO, y);

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
    let shift = (magnitude >> EXP_SHIFT) - 150;
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
fn cos_kernel(x: f64) -> f32 {
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
fn sin_kernel(x: f64) -> f32 {
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
    crate::fast_mul_add(y, x, x) as f32
}

/// Sine
#[must_use]
#[inline]
pub fn sinf(x: f32) -> f32 {
    let y = match x.abs() {
        9830.398 => -0.347_613_25,
        x if !x.is_finite() => f32::NAN,

        #[rustfmt::skip]
        x => {
            let (q, x) = rem_pio2(x);
            let s = sin_kernel(x);
            let c = cos_kernel(x);
            let y = if q & 1 == 0 { s } else { c };
            if q & 2 == 0 { y } else { -y }
        }
    };

    #[rustfmt::skip]
    return if x.is_sign_negative() { -y } else { y };
}

/// `sin(θ)/z` residual Taylor coefficients in CORE-MATH sinpif's fixed-point
/// `z` scale — verbatim `sn`.
const SINPIF_SN: [f64; 3] = [
    1.142_904_749_427_467e-11,
    -2.488_163_196_168_101e-34,
    1.625_023_320_396_236e-57,
];
/// `(cos(θ) − 1)/z²` residual coefficients — verbatim `cn`.
const SINPIF_CN: [f64; 3] = [
    -6.531_156_331_319_305e-23,
    7.109_333_835_435_933e-46,
    -3.095_411_451_319_522_5e-69,
];
/// `sin(πk/64)` for the full circle, `k = 0..=127` — CORE-MATH sinpif's `S`;
/// signs are baked in, and `S[(k + 32) & 127]` reads the cosine.
const SINPIF_S: [f64; 128] = [
    0.0,
    0.049_067_674_327_418_015,
    0.098_017_140_329_560_6,
    0.146_730_474_455_361_75,
    0.195_090_322_016_128_28,
    0.242_980_179_903_263_9,
    0.290_284_677_254_462_4,
    0.336_889_853_392_220_05,
    0.382_683_432_365_089_8,
    0.427_555_093_430_282_1,
    0.471_396_736_825_997_64,
    0.514_102_744_193_221_8,
    0.555_570_233_019_602_2,
    0.595_699_304_492_433_4,
    0.634_393_284_163_645_5,
    0.671_558_954_847_018_4,
    0.707_106_781_186_547_6,
    0.740_951_125_354_959_1,
    0.773_010_453_362_737,
    0.803_207_531_480_644_9,
    0.831_469_612_302_545_2,
    0.857_728_610_000_272_1,
    0.881_921_264_348_355,
    0.903_989_293_123_443_3,
    0.923_879_532_511_286_7,
    0.941_544_065_183_020_8,
    0.956_940_335_732_208_8,
    0.970_031_253_194_544,
    0.980_785_280_403_230_4,
    0.989_176_509_964_781,
    0.995_184_726_672_196_9,
    0.998_795_456_205_172_4,
    1.0,
    0.998_795_456_205_172_4,
    0.995_184_726_672_196_9,
    0.989_176_509_964_781,
    0.980_785_280_403_230_4,
    0.970_031_253_194_544,
    0.956_940_335_732_208_8,
    0.941_544_065_183_020_8,
    0.923_879_532_511_286_7,
    0.903_989_293_123_443_3,
    0.881_921_264_348_355,
    0.857_728_610_000_272_1,
    0.831_469_612_302_545_2,
    0.803_207_531_480_644_9,
    0.773_010_453_362_737,
    0.740_951_125_354_959_1,
    0.707_106_781_186_547_6,
    0.671_558_954_847_018_4,
    0.634_393_284_163_645_5,
    0.595_699_304_492_433_4,
    0.555_570_233_019_602_2,
    0.514_102_744_193_221_8,
    0.471_396_736_825_997_64,
    0.427_555_093_430_282_1,
    0.382_683_432_365_089_8,
    0.336_889_853_392_220_05,
    0.290_284_677_254_462_4,
    0.242_980_179_903_263_9,
    0.195_090_322_016_128_28,
    0.146_730_474_455_361_75,
    0.098_017_140_329_560_6,
    0.049_067_674_327_418_015,
    0.0,
    -0.049_067_674_327_418_015,
    -0.098_017_140_329_560_6,
    -0.146_730_474_455_361_75,
    -0.195_090_322_016_128_28,
    -0.242_980_179_903_263_9,
    -0.290_284_677_254_462_4,
    -0.336_889_853_392_220_05,
    -0.382_683_432_365_089_8,
    -0.427_555_093_430_282_1,
    -0.471_396_736_825_997_64,
    -0.514_102_744_193_221_8,
    -0.555_570_233_019_602_2,
    -0.595_699_304_492_433_4,
    -0.634_393_284_163_645_5,
    -0.671_558_954_847_018_4,
    -0.707_106_781_186_547_6,
    -0.740_951_125_354_959_1,
    -0.773_010_453_362_737,
    -0.803_207_531_480_644_9,
    -0.831_469_612_302_545_2,
    -0.857_728_610_000_272_1,
    -0.881_921_264_348_355,
    -0.903_989_293_123_443_3,
    -0.923_879_532_511_286_7,
    -0.941_544_065_183_020_8,
    -0.956_940_335_732_208_8,
    -0.970_031_253_194_544,
    -0.980_785_280_403_230_4,
    -0.989_176_509_964_781,
    -0.995_184_726_672_196_9,
    -0.998_795_456_205_172_4,
    -1.0,
    -0.998_795_456_205_172_4,
    -0.995_184_726_672_196_9,
    -0.989_176_509_964_781,
    -0.980_785_280_403_230_4,
    -0.970_031_253_194_544,
    -0.956_940_335_732_208_8,
    -0.941_544_065_183_020_8,
    -0.923_879_532_511_286_7,
    -0.903_989_293_123_443_3,
    -0.881_921_264_348_355,
    -0.857_728_610_000_272_1,
    -0.831_469_612_302_545_2,
    -0.803_207_531_480_644_9,
    -0.773_010_453_362_737,
    -0.740_951_125_354_959_1,
    -0.707_106_781_186_547_6,
    -0.671_558_954_847_018_4,
    -0.634_393_284_163_645_5,
    -0.595_699_304_492_433_4,
    -0.555_570_233_019_602_2,
    -0.514_102_744_193_221_8,
    -0.471_396_736_825_997_64,
    -0.427_555_093_430_282_1,
    -0.382_683_432_365_089_8,
    -0.336_889_853_392_220_05,
    -0.290_284_677_254_462_4,
    -0.242_980_179_903_263_9,
    -0.195_090_322_016_128_28,
    -0.146_730_474_455_361_75,
    -0.098_017_140_329_560_6,
    -0.049_067_674_327_418_015,
];

/// Sine of π·x
///
/// Verbatim port of CORE-MATH's `cr_sinpif`: an integer reduction of the
/// signed mantissa yields the 1/64-half-turn grid index and the exact
/// fixed-point residual, one 128-entry signed table covers the full circle
/// (cosine by `+32` offset), and plain-f64 residual polynomials finish with a
/// single rounding — no gate needed at f32 precision.  The exhaustive 2³²
/// sweep in `tests/sinpif.rs` certifies every input.
#[must_use]
#[inline]
pub fn sinpif(x: f32) -> f32 {
    let ix = x.to_bits();
    let e = ((ix >> 23) & 0xff) as i32;
    if e == 0xff {
        return f32::NAN; // ±∞ and NaN
    }
    let sgn = (ix as i32) >> 31;
    let m = ((((ix & (u32::MAX >> 9)) | 1 << 23) as i32) ^ sgn) - sgn;
    let s = 143 - e;
    if s < 0 {
        // |x| ≥ 2¹⁷: every value sits exactly on the 1/64 grid.
        if s < -6 {
            return f32::copysign(0.0, x); // |x| ≥ 2²³: all integers
        }
        let iq = (m as u32).wrapping_shl((-s - 1) as u32) & 127;
        if iq & 63 == 0 {
            return f32::copysign(0.0, x); // integer x
        }
        #[allow(clippy::cast_possible_truncation)]
        return SINPIF_S[iq as usize] as f32;
    }
    if s > 30 {
        // |x| < 2⁻¹⁴: sin(πx) = πx − (πx)³/6 in f64, one rounding out.
        let z = f64::from(x);
        let z2 = z * z;
        #[allow(clippy::suboptimal_flops)] // CORE-MATH's certified form
        return (z * crate::fast_mul_add(z2, -5.167_712_780_049_97, core::f64::consts::PI)) as f32;
    }
    let si = 25 - s;
    if si >= 0 && (m as u32).wrapping_shl(si as u32) == 0 {
        return f32::copysign(0.0, x); // integer x
    }

    let k = (m as u32).wrapping_shl((31 - s) as u32) as i32;
    let z = f64::from(k);
    let z2 = z * z;
    let fs = crate::fast_mul_add(
        z2,
        crate::fast_mul_add(z2, SINPIF_SN[2], SINPIF_SN[1]),
        SINPIF_SN[0],
    );
    let fc = crate::fast_mul_add(
        z2,
        crate::fast_mul_add(z2, SINPIF_CN[2], SINPIF_CN[1]),
        SINPIF_CN[0],
    );
    let iq = ((m >> s) + 1) >> 1;
    let ts = SINPIF_S[(iq & 127) as usize];
    let tc = SINPIF_S[((iq + 32) & 127) as usize];
    #[allow(clippy::suboptimal_flops)] // CORE-MATH's certified splitting
    let r = ts + (ts * z2) * fc + (tc * z) * fs;
    #[allow(clippy::cast_possible_truncation)]
    return r as f32;
}

/// Cosine of π·x
///
/// Verbatim port of CORE-MATH's `cr_cospif`, sharing [`sinpif`]'s tables: the
/// unsigned mantissa (cosine is even) reduces to the same 1/64-half-turn grid
/// with a `+32` quarter-turn shift, grid-exact inputs read the table
/// directly, and the tiny band closes with a single f32 FMA around 1.  The
/// exhaustive 2³² sweep in `tests/cospif.rs` certifies every input.
#[must_use]
#[inline]
pub fn cospif(x: f32) -> f32 {
    let ix = x.to_bits();
    let e = ((ix >> 23) & 0xff) as i32;
    if e == 0xff {
        return f32::NAN; // ±∞ and NaN
    }
    let m = (ix & (u32::MAX >> 9)) | 1 << 23;
    let s = 143 - e;
    let p = e - 112;
    if p < 0 {
        // |x| < 2⁻¹⁵: cos(πx) = 1 − (πx)²/2 in one f32 FMA; the coefficient
        // product underflows below 0x1.9f03p−129, where −x·x suffices.
        return if ix & (u32::MAX >> 1) >= 0x0019_f030 {
            crate::fmaf(-4.934_802_f32 * x, x, 1.0)
        } else {
            crate::fmaf(-x, x, 1.0)
        };
    }
    if p > 31 {
        if p > 63 {
            return 1.0; // |x| ≥ 2⁴⁷·⁻: all even integers at f32 precision
        }
        let iq = m.wrapping_shl((p - 32) as u32);
        return SINPIF_S[((iq.wrapping_add(32)) & 127) as usize] as f32;
    }
    let k = m.wrapping_shl(p as u32) as i32;
    if k == 0 {
        // Exactly on the 1/64 grid (integers and half-integers included).
        let iq = m >> (32 - p);
        return SINPIF_S[((iq.wrapping_add(32)) & 127) as usize] as f32;
    }
    let z = f64::from(k);
    let z2 = z * z;
    let fs = crate::fast_mul_add(
        z2,
        crate::fast_mul_add(z2, SINPIF_SN[2], SINPIF_SN[1]),
        SINPIF_SN[0],
    );
    let fc = crate::fast_mul_add(
        z2,
        crate::fast_mul_add(z2, SINPIF_CN[2], SINPIF_CN[1]),
        SINPIF_CN[0],
    );
    let iq = ((m >> s) + 1) >> 1;
    let ts = SINPIF_S[((iq + 32) & 127) as usize]; // cosine by quarter-turn shift
    let tc = SINPIF_S[(iq & 127) as usize];
    #[allow(clippy::suboptimal_flops)] // CORE-MATH's certified splitting
    let r = ts + (ts * z2) * fc - (tc * z) * fs;
    #[allow(clippy::cast_possible_truncation)]
    return r as f32;
}

/// Cosine
#[must_use]
#[inline]
pub fn cosf(x: f32) -> f32 {
    let x = x.abs();

    match x {
        2.861_650_8e15 => return 0.533_916_4,
        1.100_467_8e19 => return 0.996_410_1,
        1.726_998_3e20 => return 0.969_058,
        x if !x.is_finite() => return f32::NAN,
        _ => (),
    }

    let (q, x) = rem_pio2(x);
    let s = sin_kernel(x);
    let c = cos_kernel(x);
    let y = if q & 1 == 0 { c } else { s };

    if (q.wrapping_add(1)) & 2 == 0 { y } else { -y }
}

/// Compute sine and cosine simultaneously
#[must_use]
#[inline]
pub fn sincosf(x: f32) -> (f32, f32) {
    let (s, c) = match x.abs() {
        9830.398 => (-0.347_613_25, -0.937_638),
        2.861_650_8e15 => (-0.845_537_3, 0.533_916_4),
        1.100_467_8e19 => (0.084_657_6, 0.996_410_1),
        1.726_998_3e20 => (-0.246_833_34, 0.969_058),
        x if !x.is_finite() => (f32::NAN, f32::NAN),
        x => {
            let (q, x) = rem_pio2(x);
            let s = sin_kernel(x);
            let c = cos_kernel(x);
            let (s, c) = if q & 1 == 0 { (s, c) } else { (c, s) };
            let s = if q & 2 == 0 { s } else { -s };
            let c = if q.wrapping_add(1) & 2 == 0 { c } else { -c };
            (s, c)
        }
    };
    let s = if x.is_sign_negative() { -s } else { s };
    (s, c)
}

/// Odd rational `tan(πz)·(¼ − z²)/(z − z³)` numerator/denominator
/// coefficients over the reduced period — CORE-MATH tanpif's `cn`/`cd`; the
/// `(¼ − z²)` factor bakes in the poles and `(z − z³)` the zeros.
const TANPIF_CN: [f64; 4] = [
    0.785_398_163_397_448_4,
    -0.280_538_726_488_783_2,
    0.022_011_589_086_914_73,
    -0.000_231_039_590_123_269_23,
];
const TANPIF_CD: [f64; 4] = [
    1.0,
    -0.647_061_134_091_576_7,
    0.097_314_025_548_005_4,
    -0.003_226_980_548_916_333_3,
];

/// Tangent of π·x
///
/// Verbatim port of CORE-MATH's `cr_tanpif`: one odd rational in f64 over the
/// reduced period `z = x − round(x)`, with the poles and zeros carried by
/// exact factors, quarter-integers returned as ±1/±0/±∞ up front, and
/// CORE-MATH's two directed-rounding patch points kept for bit fidelity.  The
/// exhaustive 2³² sweep in `tests/tanpif.rs` certifies every input.
#[must_use]
#[inline]
pub fn tanpif(x: f32) -> f32 {
    let ix = x.to_bits();
    let e = ix & (0xff << 23);
    if e > 150 << 23 {
        if e == 0xff << 23 {
            return if ix << 9 == 0 { f32::NAN } else { x + x };
        }
        return f32::copysign(0.0, x); // |x| > 2²³: all even integers
    }
    let x4 = 4.0 * x;
    let dx4 = x4 - x4.round_ties_even();
    let zf = x - x.round_ties_even();
    if dx4 == 0.0 {
        // 4x is an integer: the exact ±1 / signed-zero / pole lattice.
        // SAFETY: |4x| ≤ 2²⁵ fits an `i32`.
        let k = unsafe { x4.to_int_unchecked::<i32>() };
        if k & 1 == 1 {
            return f32::copysign(1.0, zf); // x = ¼ mod ½
        }
        return match k & 6 {
            0 => f32::copysign(0.0, x),  // x = 0 mod 2
            4 => -f32::copysign(0.0, x), // x = 1 mod 2
            2 => f32::INFINITY,          // x = ½ mod 2
            _ => f32::NEG_INFINITY,      // x = −½ mod 2
        };
    }
    // CORE-MATH's two patch points (directed-rounding shims kept verbatim).
    let a = zf.to_bits() & (u32::MAX >> 1);
    if a == 0x3e93_3802 {
        return f32::copysign(1.268_794_7, zf) + f32::copysign(2.980_232_2e-08, zf);
    }
    if a == 0x38f2_6685 {
        return f32::copysign(0.000_363_122_73, zf) + f32::copysign(7.275_958e-12, zf);
    }

    let z = f64::from(zf);
    let z2 = z * z;
    let z4 = z2 * z2;
    #[allow(clippy::suboptimal_flops)] // CORE-MATH's certified splitting
    let r = (z - z * z2)
        * ((TANPIF_CN[0] + z2 * TANPIF_CN[1]) + z4 * (TANPIF_CN[2] + z2 * TANPIF_CN[3]))
        / (((TANPIF_CD[0] + z2 * TANPIF_CD[1]) + z4 * (TANPIF_CD[2] + z2 * TANPIF_CD[3]))
            * (0.25 - z2));
    #[allow(clippy::cast_possible_truncation)]
    return r as f32;
}

/// Tangent function
///
/// After [`rem_pio2`] reduces `x` to `y ∈ [-π/4, π/4]` with quadrant
/// `q`, evaluate `tan(y) = y·p(y²)/q(y²)` as a degree-3/3 rational.  In an odd
/// quadrant we want `-cot(y) = -q(y²)/(y·p(y²))`, the same two polynomials
/// with numerator and denominator swapped.
///
/// Coefficients (relative error ≈ 2⁻⁵¹) generated with
/// `ratapprox --function="tan(sqrt(x))/sqrt(x)" --dom="[0.0001,0.6168]"
///   --type=[3,3] --numF=D --denF=D`.
#[must_use]
#[inline]
pub fn tanf(x: f32) -> f32 {
    const NUM: [f64; 4] = [
        1.0,
        -0.128_282_401_241_495_37,
        2.805_799_105_412_74e-3,
        -7.482_480_453_622_507e-6,
    ];
    const DEN: [f64; 4] = [
        1.0,
        -0.461_615_734_574_826_74,
        2.334_437_729_696_323e-2,
        -2.084_309_371_418_349_5e-4,
    ];

    let y = match x.abs() {
        x if !x.is_finite() => f32::NAN,
        x => {
            let (q, y) = rem_pio2(x);
            let u = y * y;
            let p = crate::poly(u, &NUM);
            let d = crate::poly(u, &DEN);
            let yp = y * p;
            let result = if q & 1 == 0 { yp / d } else { -d / yp };
            result as f32
        }
    };

    #[rustfmt::skip]
    return if x.is_sign_negative() { -y } else { y };
}
