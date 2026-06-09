use super::exp::finite_exp;
use super::log::atanh as atanh_kernel;
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

/// Inverse hyperbolic sine
#[must_use]
#[inline]
pub fn asinhf(x: f32) -> f32 {
    use core::f64::consts;
    let s = x.abs();

    let magnitude = match s {
        f32::INFINITY => f32::INFINITY,
        2.901_895_4e7 => 17.876_608,
        6.391_892e22 => 53.20505,
        2.749_153e28 => 66.17683,
        s if s.is_nan() => f32::NAN,
        _ => {
            let s: f64 = s.into();
            let c = crate::fast_mul_add(s, s, 1.0).sqrt();
            let i = (c + s).to_bits() as i64;
            let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> F64_EXP_SHIFT;
            let (s, c) = if exponent == 0 {
                (s, c)
            } else {
                let c = f64::from_bits((i - (exponent << F64_EXP_SHIFT)) as u64);
                (c - 1.0, c)
            };

            crate::fast_mul_add(
                consts::LN_2,
                exponent as f64,
                2.0 * atanh_kernel(s / (c + 1.0)),
            ) as f32
        }
    };

    magnitude.copysign(x)
}

/// Inverse hyperbolic cosine
#[must_use]
#[inline]
pub fn acoshf(x: f32) -> f32 {
    match x {
        f32::INFINITY => f32::INFINITY,
        6.391_892e22 => 53.20505,
        2.749_153e28 => 66.17683,

        (1.0..) => {
            use core::f64::consts;

            let c: f64 = x.into();
            let s = crate::fast_mul_add(c, c, -1.0).sqrt();
            let i = (c + s).to_bits() as i64;
            let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> F64_EXP_SHIFT;

            let x = f64::from_bits((i - (exponent << F64_EXP_SHIFT)) as u64);

            crate::fast_mul_add(
                consts::LN_2,
                exponent as f64,
                2.0 * atanh_kernel((x - 1.0) / (x + 1.0)),
            ) as f32
        }

        _ => f32::NAN,
    }
}
