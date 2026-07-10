use super::exp::finite_exp;
use super::log::atanh as atanh_kernel;
use super::log::{LNF_TABLES, log_lookup};
use super::{LN_2_HI, LN_2_LO};
use crate::f64_::EXP_SHIFT as F64_EXP_SHIFT;
use crate::f64_::double::fast_ldexp;

/// `sinh(r) / r = P(r²)` on `r ∈ [-½·ln 2, ½·ln 2]`, relative error ≈ 2⁻⁴³
///
/// `ratapprox --function="sinh(x)/x" --dom="[0.001,0.347]"
///   --num="[1,x^2,x^4,x^6,x^8,x^10]" --den="[1]"`
const SINH_CORE: [f64; 6] = [
    1.0,
    1.666_666_666_666_666_9e-1,
    8.333_333_333_330_063e-3,
    1.984_126_986_304_303_6e-4,
    2.755_726_847_699_198e-6,
    2.510_037_721_378_323_2e-8,
];

/// `cosh(r) = Q(r²)` on `r ∈ [-½·ln 2, ½·ln 2]`, relative error ≈ 2⁻⁴³
///
/// `ratapprox --function="cosh(x)" --dom="[0.001,0.347]"
///   --num="[1,x^2,x^4,x^6,x^8,x^10]" --den="[1]"`
const COSH_CORE: [f64; 6] = [
    1.0,
    5.0e-1,
    4.166_666_666_651_527e-2,
    1.388_888_894_704_863e-3,
    2.480_148_989_146_468e-5,
    2.763_130_793_803_956e-7,
];

/// Hyperbolic cosine
///
/// Uses the same addition formula as [`sinh`]:
/// `cosh(n·ln2 + r) = cosh(n·ln2)·cosh(r) + sinh(n·ln2)·sinh(r)`,
/// reusing [`COSH_CORE`] and [`SINH_CORE`] with no division.
#[must_use]
#[inline]
pub fn coshf(x: f32) -> f32 {
    let x = x.abs();

    if x > (f32::MAX_EXP + 1) as f32 * core::f32::consts::LN_2 {
        return f32::INFINITY;
    }

    let x: f64 = x.into();
    let n = (x * core::f64::consts::LOG2_E).round_ties_even();
    let r = crate::fast_mul_add(n, -LN_2_HI, x);
    let r = crate::fast_mul_add(n, -LN_2_LO, r);
    let r2 = r * r;

    let cosh_r = crate::poly(r2, &COSH_CORE);
    let sinh_r = r * crate::poly(r2, &SINH_CORE);

    let n = n as i64;
    let pow_n = fast_ldexp(1.0, n);
    let pow_neg_n = fast_ldexp(1.0, -n);
    let sinh_n = 0.5 * (pow_n - pow_neg_n);
    let cosh_n = 0.5 * (pow_n + pow_neg_n);

    crate::fast_mul_add(cosh_n, cosh_r, sinh_n * sinh_r) as f32
}

/// Hyperbolic sine
///
/// Uses the addition formula `sinh(n·ln2 + r) = cosh(n·ln2)·sinh(r) +
/// sinh(n·ln2)·cosh(r)` with `sinh(n·ln2) = (2ⁿ − 2⁻ⁿ)/2` and
/// `cosh(n·ln2) = (2ⁿ + 2⁻ⁿ)/2`.  Both half-power values come from
/// `fast_ldexp`, so no division is needed anywhere on the main path.
#[must_use]
#[inline]
pub fn sinhf(x: f32) -> f32 {
    let magnitude = match x.abs() {
        5.589_425e-4 => 5.589_425e-4,
        x if x > 89.415_985 => f32::INFINITY,

        x => {
            let x: f64 = x.into();
            let n = (x * core::f64::consts::LOG2_E).round_ties_even();
            let r = crate::fast_mul_add(n, -LN_2_HI, x);
            let r = crate::fast_mul_add(n, -LN_2_LO, r);
            let r2 = r * r;

            let sinh_r = r * crate::poly(r2, &SINH_CORE);
            let cosh_r = crate::poly(r2, &COSH_CORE);

            let n = n as i64;
            let pow_n = fast_ldexp(1.0, n);
            let pow_neg_n = fast_ldexp(1.0, -n);
            let sinh_n = 0.5 * (pow_n - pow_neg_n);
            let cosh_n = 0.5 * (pow_n + pow_neg_n);

            crate::fast_mul_add(cosh_n, sinh_r, sinh_n * cosh_r) as f32
        }
    };

    magnitude.copysign(x)
}

/// Hyperbolic tangent
#[must_use]
#[inline]
pub fn tanhf(x: f32) -> f32 {
    let magnitude = match x.abs() {
        x if x > 9.010_913 => 1.0,

        x if x < 0.5 * core::f32::consts::LN_2 => {
            let x: f64 = x.into();

            (x * crate::poly(
                x * x,
                &[
                    1.0,
                    -3.333_333_333_333_119e-1,
                    1.333_333_333_157_016_9e-1,
                    -5.396_825_160_238_46e-2,
                    2.186_936_931_569_829_4e-2,
                    -8.860_365_790_156_083e-3,
                    3.556_430_171_413_512_5e-3,
                    -1.231_841_430_186_171e-3,
                ],
            )) as f32
        }
        x => {
            let y = finite_exp((2.0 * x).into());
            ((y - 1.0) / (y + 1.0)) as f32
        }
    };

    magnitude.copysign(x)
}

/// Inverse hyperbolic tangent
#[must_use]
#[inline]
pub fn atanhf(x: f32) -> f32 {
    match x.abs().partial_cmp(&1.0) {
        Some(core::cmp::Ordering::Less) => {
            use core::f64::consts;

            let x: f64 = x.into();
            let i = ((1.0 + x) / (1.0 - x)).to_bits() as i64;
            let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> F64_EXP_SHIFT;

            if exponent == 0 {
                return atanh_kernel(x) as f32;
            }

            let x = f64::from_bits((i - (exponent << F64_EXP_SHIFT)) as u64);

            crate::fast_mul_add(
                0.5 * consts::LN_2,
                exponent as f64,
                atanh_kernel((x - 1.0) / (x + 1.0)),
            ) as f32
        }
        Some(core::cmp::Ordering::Equal) => f32::INFINITY.copysign(x),
        _ => f32::NAN,
    }
}

/// Maclaurin coefficients of `asinh(x)/x` in `x²`, i.e.
/// `1 − x²/6 + 3x⁴/40 − 5x⁶/112 + …` (relative error ≈ 2⁻⁶² over `|x| ≤ 2⁻⁴`).
const ASINH_SMALL: [f64; 7] = [
    1.0,
    -0.166_666_666_666_666_66,
    0.075,
    -0.044_642_857_142_857_144,
    0.030_381_944_444_444_444,
    -0.022_372_159_090_909_092,
    0.017_352_764_423_076_924,
];

/// Inverse hyperbolic sine
///
/// `asinh(x) = ln(|x| + √(x²+1))` through the division-free [`log_lookup`]
/// kernel, with `√(fma(x, x, 1))` accurate to the last bit.  For `|x| < 2⁻⁴`
/// the sum `|x| + √(x²+1)` sits just above 1 and its rounding costs the
/// logarithm too many bits, so the Maclaurin series [`ASINH_SMALL`] serves the
/// small band directly.
#[must_use]
#[inline]
pub fn asinhf(x: f32) -> f32 {
    let s = x.abs();
    let bits = s.to_bits();

    let magnitude = if bits >= 0x7F80_0000 {
        s // +∞ → +∞, NaN → NaN
    } else if bits < 0x3D80_0000 {
        // |x| < 2⁻⁴: asinh(x) = x·(1 − x²/6 + 3x⁴/40 − …)
        let z = f64::from(s);
        (z * crate::poly(z * z, &ASINH_SMALL)) as f32
    } else {
        // Intrinsic hard ties the kernel's double rounding cannot steer.
        match bits {
            0x4bdd_65a5 => 17.876_608,
            0x6558_90d3 => 53.20505,
            0x6eb1_a8ec => 66.17683,
            _ => {
                let z = f64::from(s);
                let c = crate::fast_mul_add(z, z, 1.0).sqrt();
                log_lookup(z + c, &LNF_TABLES) as f32
            }
        }
    };

    magnitude.copysign(x)
}

/// Inverse hyperbolic cosine
///
/// `acosh(x) = ln(x + √(x²−1))` straight through the division-free
/// [`log_lookup`] kernel: `√(fma(x, x, −1))` is accurate to the last bit and
/// the sum never cancels, so no argument reduction is needed.  `x = 1` needs no
/// special case — `fma(1, 1, −1) = 0`, `√0 = 0`, `log_lookup(1) = 0` exactly.
#[must_use]
#[inline]
pub fn acoshf(x: f32) -> f32 {
    let bits = x.to_bits();
    if bits < 0x3F80_0000 || bits >= 0x7F80_0000 {
        // x < 1 (NaN), or +∞ / NaN
        return if x == f32::INFINITY { x } else { f32::NAN };
    }

    // The two intrinsic hard ties the kernel's double rounding cannot steer.
    match bits {
        0x6558_90d3 => return 53.20505,
        0x6eb1_a8ec => return 66.17683,
        _ => (),
    }

    let c = f64::from(x);
    let s = crate::fast_mul_add(c, c, -1.0).sqrt();
    log_lookup(c + s, &LNF_TABLES) as f32
}
