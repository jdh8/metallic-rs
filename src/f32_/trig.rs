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
