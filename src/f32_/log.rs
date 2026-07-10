use crate::f64_::EXP_SHIFT as F64_EXP_SHIFT;

/// Polynomial approximation of inverse hyperbolic tangent restricted to
/// `-c..=c`, where
///
/// ```text
///     √2 - 1                  1 + c
/// c = ------  the solution to ----- = √2.
///     √2 + 1,                 1 - c
/// ```
#[inline]
pub(super) fn atanh(x: f64) -> f64 {
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
    crate::fast_mul_add(y, x, x)
}

/// Natural logarithm
#[must_use]
#[inline]
pub fn logf(x: f32) -> f32 {
    match super::normalize(x) {
        (crate::Sign::Positive, super::Magnitude::Infinite) => f32::INFINITY,
        (_, super::Magnitude::Zero) => f32::NEG_INFINITY,
        (crate::Sign::Negative, _) | (_, super::Magnitude::Nan) => f32::NAN,

        (crate::Sign::Positive, super::Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;

            // TODO: these hard-coded returns are correctly-rounded values for
            // double-rounding cases (the exact ln is within ~½ f64-ulp of an f32
            // midpoint, so the f64 kernel rounds the wrong way).  Remove them by
            // rounding the f64 result to odd / carrying a hi+lo pair before the
            // final f32 round, as the f64 logarithm functions now do.
            match x {
                1.179_438_3e-2 => return -4.440_131_7,
                9.472_636 => return 2.248_407_1,
                5.803_790_8e7 => return 17.876_608,
                1.278_378_4e23 => return 53.20505,
                5.498_306e28 => return 66.17683,
                _ => (),
            }

            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> super::EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << super::EXP_SHIFT)) as u32).into();

            crate::fast_mul_add(
                core::f64::consts::LN_2,
                exponent.into(),
                2.0 * atanh((x - 1.0) / (x + 1.0)),
            ) as f32
        }
    }
}

/// Compute `ln(1 + x)` accurately especially for small `x`
#[must_use]
#[inline]
pub fn log1pf(x: f32) -> f32 {
    // TODO: the hard-coded returns below are double-rounding cases (see the note
    // in `ln`); remove them with a round-to-odd / hi+lo final round.
    match x {
        f32::INFINITY => f32::INFINITY,
        -1.0 => f32::NEG_INFINITY,
        -2.178_714_6e-3 => -2.181_091_6e-3,
        -8.583_044e-6 => -8.583_081e-6,
        -7.152_555_7e-7 => -7.152_558e-7,
        7.152_559e-7 => 7.152_557e-7,
        8.583_093e-6 => 8.583_057e-6,
        0.495_129_97 => 0.402_213_13,
        8.472_636 => 2.248_407_1,
        1.278_378_4e23 => 53.20505,
        5.498_306e28 => 66.17683,
        x if x < -1.0 || x.is_nan() => f32::NAN,
        _ => {
            use core::f64::consts::FRAC_1_SQRT_2;
            let x: f64 = x.into();
            let i = (1.0 + x).to_bits() as i64;
            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i64) >> F64_EXP_SHIFT;
            let y = f64::from_bits((i - (exponent << F64_EXP_SHIFT)) as u64);
            let z = if exponent == 0 { x } else { y - 1.0 };

            crate::fast_mul_add(
                -core::f64::consts::LN_2,
                -exponent as f64,
                2.0 * atanh(z / (z + 2.0)),
            ) as f32
        }
    }
}

/// Base 2 logarithm
#[must_use]
#[inline]
pub fn log2f(x: f32) -> f32 {
    match super::normalize(x) {
        (crate::Sign::Positive, super::Magnitude::Infinite) => f32::INFINITY,
        (_, super::Magnitude::Zero) => f32::NEG_INFINITY,
        (crate::Sign::Negative, _) | (_, super::Magnitude::Nan) => f32::NAN,

        (crate::Sign::Positive, super::Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;
            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> super::EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << super::EXP_SHIFT)) as u32).into();

            crate::fast_mul_add(
                2.0 * core::f64::consts::LOG2_E,
                atanh((x - 1.0) / (x + 1.0)),
                exponent.into(),
            ) as f32
        }
    }
}

/// Base 10 logarithm
#[must_use]
#[inline]
pub fn log10f(x: f32) -> f32 {
    const LOG10_2_HI: f64 = 0.301_029_995_663_981_25;
    const LOG10_2_LO: f64 = -5.831_487_935_904_3e-17;

    if x.eq(&6.284_548e-30) {
        return -29.201_727;
    }

    match super::normalize(x) {
        (crate::Sign::Positive, super::Magnitude::Infinite) => f32::INFINITY,
        (_, super::Magnitude::Zero) => f32::NEG_INFINITY,
        (crate::Sign::Negative, _) | (_, super::Magnitude::Nan) => f32::NAN,

        (crate::Sign::Positive, super::Magnitude::Normalized(i)) => {
            use core::f32::consts::FRAC_1_SQRT_2;
            use core::f64::consts;

            let exponent = (i - FRAC_1_SQRT_2.to_bits() as i32) >> super::EXP_SHIFT;
            let x: f64 = f32::from_bits((i - (exponent << super::EXP_SHIFT)) as u32).into();
            let x = crate::fast_mul_add(
                2.0 * consts::LOG10_E,
                atanh((x - 1.0) / (x + 1.0)),
                LOG10_2_LO * f64::from(exponent),
            );
            crate::fast_mul_add(LOG10_2_HI, exponent.into(), x) as f32
        }
    }
}

/// Binary logarithm of 1 plus `x`
///
/// One `f64` pass: `1 + x` is exact (an f32 mantissa plus the unit bit spans
/// at most 53 bits), the exponent splits off the [`log2f`] way — an integer
/// subtract centred on √2/2 on the `f64` bit pattern — and the shared
/// [`atanh`] kernel finishes.  `s − 1` is Sterbenz-exact in the unreduced
/// binade, so the cancellation near `x → 0` is benign; below `2⁻³⁰` the
/// two-term series avoids forming `1 + x` at all (its `x³/3` tail is under
/// `2⁻⁶¹` relative).  The exhaustive 2³² sweep in `tests/log2p1f.rs`
/// certifies every input.
#[must_use]
#[inline]
pub fn log2p1f(x: f32) -> f32 {
    use core::f64::consts::LOG2_E;

    if x == 0.0 || x.is_nan() || x == f32::INFINITY {
        return x;
    }
    if x < -1.0 {
        return f32::NAN;
    }
    if x == -1.0 {
        return f32::NEG_INFINITY;
    }

    // The two f32 ties the kernel's double rounding cannot steer.
    if x.to_bits() == 0x4ebd_09e3 {
        return f32::from_bits(0x41f4_8013);
    }
    if x.to_bits() == 0x5292_8e33 {
        return f32::from_bits(0x4218_c7fd);
    }

    let xd = f64::from(x);
    if x.abs() < 1.862_645_149_230_957e-9 {
        // |x| < 2⁻²⁹: 1 + x would round (an f32 mantissa this low spans past
        // bit −52), and the x³/3 series tail is below 2⁻⁵⁹ relative.
        return (LOG2_E * crate::fast_mul_add(-0.5 * xd, xd, xd)) as f32;
    }

    let s = 1.0 + xd;
    let i = s.to_bits() as i64;
    #[allow(clippy::cast_possible_truncation)] // |exponent| ≤ 128
    let exponent =
        ((i - core::f64::consts::FRAC_1_SQRT_2.to_bits() as i64) >> F64_EXP_SHIFT) as i32;
    #[allow(clippy::cast_sign_loss)] // the shifted-back pattern is a positive normal
    let m = f64::from_bits((i - (i64::from(exponent) << F64_EXP_SHIFT)) as u64);

    crate::fast_mul_add(2.0 * LOG2_E, atanh((m - 1.0) / (m + 1.0)), exponent.into()) as f32
}

/// Common logarithm of 1 plus `x`
///
/// [`log2p1f`]'s reduction with [`log10f`]'s base-10 finish: the exponent
/// term carries `log₁₀2` as a two-word constant so its rounding stays below
/// the kernel's accuracy.  The exhaustive 2³² sweep in `tests/log10p1f.rs`
/// certifies every input.
#[must_use]
#[inline]
pub fn log10p1f(x: f32) -> f32 {
    const LOG10_2_HI: f64 = 0.301_029_995_663_981_25;
    const LOG10_2_LO: f64 = -5.831_487_935_904_3e-17;
    use core::f64::consts::LOG10_E;

    if x == 0.0 || x.is_nan() || x == f32::INFINITY {
        return x;
    }
    if x < -1.0 {
        return f32::NAN;
    }
    if x == -1.0 {
        return f32::NEG_INFINITY;
    }

    // The one f32 tie the series leg's double rounding cannot steer.
    if x.to_bits() == 0xb051_e173 {
        return f32::from_bits(0xafb6_4ccf);
    }

    let xd = f64::from(x);
    if x.abs() < 1.862_645_149_230_957e-9 {
        // |x| < 2⁻²⁹: see `log2p1f`
        return (LOG10_E * crate::fast_mul_add(-0.5 * xd, xd, xd)) as f32;
    }

    let s = 1.0 + xd;
    let i = s.to_bits() as i64;
    #[allow(clippy::cast_possible_truncation)] // |exponent| ≤ 128
    let exponent =
        ((i - core::f64::consts::FRAC_1_SQRT_2.to_bits() as i64) >> F64_EXP_SHIFT) as i32;
    #[allow(clippy::cast_sign_loss)] // the shifted-back pattern is a positive normal
    let m = f64::from_bits((i - (i64::from(exponent) << F64_EXP_SHIFT)) as u64);

    let y = crate::fast_mul_add(
        2.0 * LOG10_E,
        atanh((m - 1.0) / (m + 1.0)),
        LOG10_2_LO * f64::from(exponent),
    );
    crate::fast_mul_add(LOG10_2_HI, exponent.into(), y) as f32
}
