use crate::f64_::double::{DoubleDouble, round};

/// Minimax coefficients of `P` in `asin(t) = t + t³·P(t²)` on `t² ∈ [0, ¼]`.
///
/// Shared by the `|x| < ½` branch of [`asin`] and by both branches of [`acos`].
/// For `|x| ≥ ½`, reflecting through `s = √((1−|x|)/2) ∈ [0, ½]` keeps the
/// kernel argument in `[0, ¼]`, so the same `P` serves the whole domain:
/// `asin(x) = π/2 − 2·asin(s)` and `acos(x) = 2·asin(s)` (or `π − 2·asin(s)`).
///
/// Relative error ≈ 2⁻⁴⁶, generated with
/// `ratapprox --function="(asin(sqrt(x))-sqrt(x))/(x*sqrt(x))"
///   --dom="[1e-30,0.25]" --num=[1,x,x^2,...,x^10] --den=[1]`.
const ASIN_NEAR_ZERO: [f64; 11] = [
    0.166_666_666_666_669_68,
    0.074_999_999_997_106_53,
    0.044_642_857_599_726_166,
    0.030_381_916_397_495_97,
    0.022_373_038_603_277_58,
    0.017_336_763_281_738_436,
    0.014_144_868_382_976_405,
    0.010_267_430_424_776_843,
    0.015_521_003_704_904_501,
    -0.006_965_450_747_007_442,
    0.027_986_438_522_030_702,
];

/// `asin(t)` for `t ∈ [0, ½]`, where `u = t²`, via `t + t³·P(t²)`.
#[inline]
fn asin_near_zero(t: f64, u: f64) -> f64 {
    crate::fast_mul_add(t * u, crate::poly(u, &ASIN_NEAR_ZERO), t)
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
const FRAC_PI_2: DoubleDouble = DoubleDouble {
    high: 1.570_796_326_794_896_6,
    low: 6.123_233_995_736_766e-17,
};

/// π as a double-double
const PI: DoubleDouble = DoubleDouble {
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
fn atan_dd(q: DoubleDouble) -> DoubleDouble {
    let k = (q.high * 8.0).round_ties_even();
    let c = k * 0.125;

    // `q.high ∈ [0, 1]` ⇒ `k ∈ {0, …, 8}`, always a valid index
    let (high, low) = ATAN_TABLE[k as usize];

    // u = (q - c) / (1 + q·c), exact in double-double (c is a power-of-two multiple)
    let u = (q + DoubleDouble { high: -c, low: 0.0 })
        * (q * c
            + DoubleDouble {
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

    DoubleDouble { high, low }
        + u
        + DoubleDouble {
            high: -cubic.high,
            low: -cubic.low,
        }
        + DoubleDouble {
            high: tail,
            low: 0.0,
        }
}

/// Magnitude of `atan2(y, x)` for finite nonzero `a = |x|`, `b = |y|`
///
/// Returns the correctly-rounded angle in `[0, π]`; the caller restores the
/// sign of `y` with `copysign`.
#[inline]
fn atan2_mag(a: f64, b: f64, x_negative: bool) -> f32 {
    let phi = if a >= b {
        atan_dd(DoubleDouble::from_quotient(b, a))
    } else {
        // π/2 - atan(a/b)
        let t = atan_dd(DoubleDouble::from_quotient(a, b));
        FRAC_PI_2
            + DoubleDouble {
                high: -t.high,
                low: -t.low,
            }
    };

    let theta = if x_negative {
        // π - φ
        PI + DoubleDouble {
            high: -phi.high,
            low: -phi.low,
        }
    } else {
        phi
    };

    round(theta)
}

/// Arcsine in half-turns
///
/// Promotes to the correctly rounded f64 [`crate::f64_::asinpi`] and rounds
/// once more to f32; the exhaustive 2³² sweep in `tests/asinpif.rs` certifies
/// the double rounding never lands on the wrong side of an f32 boundary.
#[must_use]
#[inline]
pub fn asinpif(x: f32) -> f32 {
    crate::f64_::asinpi(x.into()) as f32
}

/// Arccosine
///
/// `acos(x) = π/2 − asin(x)`.  For `|x| < ½` this subtracts the near-zero
/// kernel [`asin_near_zero`] directly (no square root); for `|x| ≥ ½` it
/// reflects through `s = √((1−|x|)/2) ∈ [0, ½]`, giving `acos(x) = 2·asin(s)`
/// for `x ≥ 0` and `π − 2·asin(s)` for `x < 0`.  Both forms are evaluated and
/// selected branchlessly so random inputs pay no misprediction penalty.
#[must_use]
#[inline]
pub fn acosf(x: f32) -> f32 {
    let xf: f64 = x.into();
    let a = xf.abs();

    // |x| < ½:  π/2 − asin(x)
    let near = core::f64::consts::FRAC_PI_2 - asin_near_zero(xf, xf * xf);

    // |x| ≥ ½:  2·asin(s) folded about π for x < 0, s = √((1−|x|)/2)
    let u = 0.5 * (1.0 - a);
    let two = 2.0 * asin_near_zero(u.sqrt(), u);
    let far = if xf.is_sign_positive() {
        two
    } else {
        core::f64::consts::PI - two
    };

    let y = (if a < 0.5 { near } else { far }) as f32;

    // Hard-to-round cases near π/2 (tiny x, near-zero branch).
    match x {
        1.589_325_5e-8 => 1.570_796_4,
        2.486_864_7e-4 => 1.570_547_7,
        _ => y,
    }
}

/// Arcsine
///
/// For `|x| < ½`, evaluate the near-zero kernel [`asin_near_zero`] directly.
/// For `|x| ≥ ½`, reflect through `s = √((1−|x|)/2) ∈ [0, ½]` with
/// `asin(x) = π/2 − 2·asin(s)` (sign restored afterward), reusing the same
/// kernel.  Both forms are evaluated and selected branchlessly.
#[must_use]
#[inline]
pub fn asinf(x: f32) -> f32 {
    let xf: f64 = x.into();
    let a = xf.abs();

    // |x| < ½:  asin(x) directly
    let near = asin_near_zero(xf, xf * xf);

    // |x| ≥ ½:  ±(π/2 − 2·asin(s)), s = √((1−|x|)/2)
    let u = 0.5 * (1.0 - a);
    let two = 2.0 * asin_near_zero(u.sqrt(), u);
    let far = if xf.is_sign_positive() {
        core::f64::consts::FRAC_PI_2 - two
    } else {
        two - core::f64::consts::FRAC_PI_2
    };

    let y = (if a < 0.5 { near } else { far }) as f32;

    // Hard-to-round case in the reflection branch.
    match x.abs() {
        0.532_136_56 => 0.561_122_06_f32.copysign(x),
        _ => y,
    }
}

/// Arctangent
#[must_use]
#[inline]
pub fn atanf(x: f32) -> f32 {
    #[inline]
    fn kernel(x: f64) -> f64 {
        fast_polynomial::rational_array(
            x * x,
            &[
                3.300_049_005_002_112e-1,
                8.269_936_280_545_194e-1,
                7.536_692_262_484_512e-1,
                3.041_250_192_035_205_3e-1,
                5.258_546_450_061_43e-2,
                3.092_811_576_351_314e-3,
                2.668_044_628_603_543_2e-5,
            ],
            &[
                3.300_049_005_002_111_4e-1,
                9.369_952_615_545_891e-1,
                1.0,
                4.972_028_574_382_380_6e-1,
                1.155_090_051_164_766_6e-1,
                1.090_224_520_186_812_4e-2,
                2.732_269_307_955_130_4e-4,
            ],
        )
    }

    let use_outer = x.abs() > 1.0;
    let x: f64 = x.into();

    if use_outer {
        use core::f64::consts::FRAC_PI_2;
        let recip = x.recip();
        crate::fast_mul_add(-recip, kernel(recip), FRAC_PI_2.copysign(x)) as f32
    } else {
        (x * kernel(x)) as f32
    }
}

/// Arctangent of `y / x` using the signs of both to select the quadrant
///
/// The result is the angle in `(-π, π]` between the positive `x`-axis and the
/// point `(x, y)`, carrying the sign of `y`.
#[must_use]
#[inline]
pub fn atan2f(y: f32, x: f32) -> f32 {
    use core::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    /// Correctly-rounded `f32` of 3π/4
    const THREE_PI_4: f32 = 2.356_194_5;

    if x.is_nan() || y.is_nan() {
        return f32::NAN;
    }

    if x.is_infinite() || y.is_infinite() {
        let magnitude = match (x.is_infinite(), y.is_infinite()) {
            (true, true) if x.is_sign_positive() => FRAC_PI_4,
            (true, true) => THREE_PI_4,
            (true, false) if x.is_sign_positive() => 0.0,
            (true, false) => PI,
            (false, true) => FRAC_PI_2,
            (false, false) => unreachable!(),
        };
        return magnitude.copysign(y);
    }

    if y == 0.0 {
        return if x.is_sign_positive() { 0.0 } else { PI }.copysign(y);
    }

    if x == 0.0 {
        return FRAC_PI_2.copysign(y);
    }

    atan2_mag(x.abs().into(), y.abs().into(), x.is_sign_negative()).copysign(y)
}
