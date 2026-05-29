#![allow(clippy::pedantic, clippy::approx_constant)]
#![warn(clippy::unreadable_literal)]

mod exp_consts;
mod kernel;
mod log_consts;
use crate::Sign;
use core::{f64, num::FpCategory};
use kernel::Sum;

/// Explicitly stored significand bits in [`prim@f64`]
///
/// This constant is usually used as a shift to access the exponent bits.
pub const EXP_SHIFT: u32 = f64::MANTISSA_DIGITS - 1;

/// Magnitude of `f64`
///
/// Nonzero subnormal numbers are normalized to have an implicit leading bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Magnitude {
    /// NaN, see [`FpCategory::Nan`]
    Nan,

    /// Infinity, see [`FpCategory::Infinite`]
    Infinite,

    /// Zero, see [`FpCategory::Zero`]
    ///
    /// Zero cannot be normalized.  A normalized magnitude has an implicit
    /// leading bit.
    Zero,

    /// Normalized magnitude
    ///
    /// The layout of the bits is the same as a normal positive `f64`.  For
    /// subnormal numbers, the stored exponent becomes zero or negative while
    /// the significand is normalized to have an implicit leading bit.
    Normalized(i64),
}

/// Break a `f64` into its sign and magnitude
#[inline]
const fn normalize(x: f64) -> (Sign, Magnitude) {
    let sign = if x.is_sign_negative() {
        Sign::Negative
    } else {
        Sign::Positive
    };
    let magnitude = x.abs().to_bits() as i64;

    match x.classify() {
        FpCategory::Nan => (sign, Magnitude::Nan),
        FpCategory::Infinite => (sign, Magnitude::Infinite),
        FpCategory::Zero => (sign, Magnitude::Zero),
        FpCategory::Normal => (sign, Magnitude::Normalized(magnitude)),
        FpCategory::Subnormal => {
            const EXPONENT_DIGITS: u32 = 64 - f64::MANTISSA_DIGITS;
            let shift = magnitude.leading_zeros() as i64 - EXPONENT_DIGITS as i64;
            let magnitude = (magnitude << shift) - (shift << EXP_SHIFT);
            (sign, Magnitude::Normalized(magnitude))
        }
    }
}

/// Rounds half-way cases away from zero
#[must_use]
#[inline]
pub fn round(x: f64) -> f64 {
    let r = x.abs();
    let i = r.trunc();

    (i + f64::from(r - i >= 0.5)).copysign(x)
}

/// The cube root
#[must_use]
#[inline]
pub fn cbrt(x: f64) -> f64 {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return x; // 0, ±inf, nan
    };

    // Scale extreme |x| into [1e-200, 1e200] so the double-double refinement keeps
    // full precision (tiny `x` loses it, huge `x` overflows `2·y³`); undo afterwards.
    // The 999 shift mirrors the `2^±999` scaling of `x`; `333 = 999/3` rescales `y`.
    let (x, magnitude, coefficient) = if x.abs() < 1e-200 {
        (
            crate::exp2i(999) * x,
            magnitude + (999 << EXP_SHIFT),
            crate::exp2i(-333),
        )
    } else if x.abs() <= 1e200 {
        (x, magnitude, 1.0)
    } else {
        (
            crate::exp2i(-999) * x,
            magnitude - (999 << EXP_SHIFT),
            crate::exp2i(333),
        )
    };

    let magnitude = (0x2A9F_7AF1_96E8_E6E8 + magnitude / 3) as u64;
    let y = f64::from_bits(crate::u64_sign_bit(sign) | magnitude);
    let y = crate::mul_add(1.0 / 3.0, x / (y * y) - y, y);
    let y = crate::mul_add(1.0 / 3.0, x / (y * y) - y, y);
    let y = y * (0.5 + 1.5 * x / crate::mul_add(2.0 * y, y * y, x));

    let quotient = Sum::from_quotient(x, y) / y;
    let sum = kernel::fast_sum(2.0 * y, quotient.high);
    let sum = Sum {
        high: sum.high,
        low: quotient.low + sum.low,
    } / 3.0;

    coefficient * (sum.high + sum.low)
}

/// Table size for the exponential family: `2`<sup>`q`</sup>` · 2`<sup>`j/N`</sup>` · exp(r)`
const EXP_N: i64 = 128;

/// The mantissa `2`<sup>`j/N`</sup>` · exp(r)` of the exponential family, as a
/// double-double normalized into [1, 2), with `q` adjusted for that normalization.
///
/// `r` is the reduced argument, `|r| ≤ ln2/2N`.  The full value is `2`<sup>`q`</sup>
/// times the returned mantissa.
#[inline]
fn exp_mantissa(j: usize, q: i64, r: Sum) -> (Sum, i64) {
    use exp_consts::{EXP2_TABLE, EXP_R_COEFFS};

    // exp(r) by double-double Horner over the degree-8 minimax polynomial.
    let (high, low) = EXP_R_COEFFS[EXP_R_COEFFS.len() - 1];
    let mut acc = Sum { high, low };

    for &(high, low) in EXP_R_COEFFS[..EXP_R_COEFFS.len() - 1].iter().rev() {
        acc = acc * r + Sum { high, low };
    }

    // Fold the table entry in and normalize the mantissa.
    let (high, low) = EXP2_TABLE[j];
    let scaled = Sum { high, low } * acc;
    let product = kernel::fast_sum(scaled.high, scaled.low);

    // `2^(j/N) · exp(r)` lies in [0.997, 2.005); fold its exponent into `q` so the
    // mantissa is in [1, 2).
    if product.high < 1.0 {
        (product * 2.0, q - 1)
    } else if product.high >= 2.0 {
        (product * 0.5, q + 1)
    } else {
        (product, q)
    }
}

/// Reconstruct `2`<sup>`q`</sup>` · 2`<sup>`j/N`</sup>` · exp(r)` for the exponential family.
///
/// `r` is the reduced argument as a double-double, `|r| ≤ ln2/2N`.  The result is
/// correctly rounded, including gradual underflow into the subnormal range.
#[inline]
fn exp_reconstruct(j: usize, q: i64, r: Sum) -> f64 {
    // The mantissa is in [1, 2), so the result is subnormal exactly when
    // `q < −1022`, and the integer-grid shift below stays within an exact `i64`.
    let (product, q) = exp_mantissa(j, q, r);

    if q >= -1022 {
        // Normal result: scaling by 2^q is exact, so one rounding of the pair.
        return kernel::fast_ldexp(product.high + product.low, q);
    }

    // Subnormal result: rounding the pair to `f64` and then scaling would round
    // twice.  Instead round the double-double on the integer grid at scale
    // 2^-1074 (the subnormal ulp): `m = (high + low) · 2^(q + 1074)` lies in
    // [0, 2^52], round it once to an integer, then `n · 2^-1074` is exact.
    let shift = q + 1074;
    let high = kernel::fast_ldexp(product.high, shift);
    let low = kernel::fast_ldexp(product.low, shift);

    // `high` may carry a half-integer resolution at this scale, so `high + low`
    // would discard the fine part of `low`.  Round `high`, then correct with the
    // exact residual `(high − n0) + low`.
    let n0 = high.round_ties_even();
    let n = n0 + ((high - n0) + low).round_ties_even();

    // 2^-1074 is the smallest positive subnormal, i.e. `f64::from_bits(1)`.
    n * f64::from_bits(1)
}

/// Argument reduction for the exponential family.
///
/// Returns `(j, q, r)` with `x = (q·N + j)·ln2/N + r` and `|r| ≤ ln2/2N`.  The
/// caller must keep `|x|` small enough that `round(N·x/ln2)` fits an `i64`
/// (`|x| < ~746`, the finite exp range).
#[inline]
fn exp_reduce(x: f64) -> (usize, i64, Sum) {
    use exp_consts::{LN2_OVER_N_HI, LN2_OVER_N_LO};

    /// `N / ln(2)`, the scale that maps `x` to the reduction index
    const N_OVER_LN2: f64 = 184.664_965_233_787_3;

    let scaled = (x * N_OVER_LN2).round_ties_even();

    // SAFETY: `|x| < 746`, so `|scaled| < 2^18`.
    let n = unsafe { scaled.to_int_unchecked::<i64>() };
    let j = (n & (EXP_N - 1)) as usize;
    let q = n >> 7;

    // r as a double-double.  `scaled · LN2_OVER_N_HI` is exact because the high
    // word has 17 trailing zero bits, and the low word recovers the tail.
    let a = scaled.mul_add(-LN2_OVER_N_HI, x);
    (j, q, Sum::from_sum(a, scaled * -LN2_OVER_N_LO))
}

/// `eˣ` as `2`<sup>`q`</sup>` · mantissa` with the mantissa a double-double in [1, 2).
///
/// The caller must ensure `x` is finite and within the non-overflow range
/// (`|x| < ~710`); used by the hyperbolic functions, which need the extra words.
#[inline]
fn exp_dd(x: f64) -> (Sum, i64) {
    let (j, q, r) = exp_reduce(x);
    exp_mantissa(j, q, r)
}

/// The exponential function
#[must_use]
#[inline]
pub fn exp(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }

    // `ln(f64::MAX)` and the threshold below which `exp` rounds to zero
    if x >= 709.782_712_893_384 {
        return f64::INFINITY;
    }
    if x <= -745.133_219_101_941 {
        return 0.0;
    }

    let (j, q, r) = exp_reduce(x);
    exp_reconstruct(j, q, r)
}

/// 2 raised to the power `x`
#[must_use]
#[inline]
pub fn exp2(x: f64) -> f64 {
    use exp_consts::{LN2_OVER_N_HI, LN2_OVER_N_LO};

    if x.is_nan() {
        return x;
    }

    // `f64::MAX_EXP` and the threshold below which `exp2` rounds to zero
    if x >= 1024.0 {
        return f64::INFINITY;
    }
    if x <= -1075.0 {
        return 0.0;
    }

    // Argument reduction: m = round(N·x), so 2^x = 2^(m/N) · 2^s with
    // s = x − m/N ∈ [−1/2N, 1/2N].  `sigma = N·x − m` is exact (N is a power of
    // two), and r = s·ln2 = sigma·(ln2/N) is carried as a double-double.
    let scaled = (x * EXP_N as f64).round_ties_even();

    // SAFETY: `|x| < 1075`, so `|scaled| < 2^18`.
    let m = unsafe { scaled.to_int_unchecked::<i64>() };
    let j = (m & (EXP_N - 1)) as usize;
    let q = m >> 7;

    let sigma = x.mul_add(EXP_N as f64, -scaled);
    let product = Sum::from_product(sigma, LN2_OVER_N_HI);
    let r = Sum {
        high: product.high,
        low: sigma.mul_add(LN2_OVER_N_LO, product.low),
    };

    exp_reconstruct(j, q, r)
}

/// 10 raised to the power `x`
#[must_use]
#[inline]
pub fn exp10(x: f64) -> f64 {
    use exp_consts::{LN10_HI, LN10_LO, LN2_OVER_N_HI, LN2_OVER_N_LO, N_LOG2_10};

    if x.is_nan() {
        return x;
    }

    // `log10(f64::MAX)` and the threshold below which `exp10` rounds to zero
    if x >= 308.254_715_559_916_7 {
        return f64::INFINITY;
    }
    if x <= -323.607_245_338_779_8 {
        return 0.0;
    }

    // 10^x = exp(x·ln10) = 2^q · 2^(j/N) · exp(r), with the reduction index
    // n = round(N·x·log2(10)) and r = x·ln10 − n·ln2/N carried as a double-double.
    let scaled = (x * N_LOG2_10).round_ties_even();

    // SAFETY: `|x| < 324`, so `|scaled| < 2^18`.
    let m = unsafe { scaled.to_int_unchecked::<i64>() };
    let j = (m & (EXP_N - 1)) as usize;
    let q = m >> 7;

    let x_ln10 = Sum::from_product(x, LN10_HI);
    let x_ln10 = Sum {
        high: x_ln10.high,
        low: x.mul_add(LN10_LO, x_ln10.low),
    };
    let n_ln2 = Sum::from_product(scaled, LN2_OVER_N_HI);
    let n_ln2 = Sum {
        high: -n_ln2.high,
        low: scaled.mul_add(-LN2_OVER_N_LO, -n_ln2.low),
    };

    exp_reconstruct(j, q, x_ln10 + n_ln2)
}

/// Compute `exp(x) − 1` accurately, especially for small `x`
#[must_use]
#[inline]
pub fn exp_m1(x: f64) -> f64 {
    use exp_consts::{LN2_OVER_N_HI, LN2_OVER_N_LO};

    /// `N / ln(2)`, the scale that maps `x` to the reduction index
    const N_OVER_LN2: f64 = 184.664_965_233_787_3;

    if x.is_nan() || x == 0.0 {
        // Preserve the sign of zero: exp_m1(±0) = ±0.
        return x;
    }

    if x >= 709.782_712_893_384 {
        return f64::INFINITY;
    }

    // Below this `exp(x) < 2^-54`, so `exp(x) − 1` rounds to exactly −1.  This also
    // keeps the reconstruction below in the normal range (`q ≥ −1022`).
    if x <= -708.0 {
        return -1.0;
    }

    // Same reduction as `exp`: n = round(N·x / ln2), r = x − n·ln2/N.
    let scaled = (x * N_OVER_LN2).round_ties_even();

    // SAFETY: `|x| < 710`, so `|scaled| < 2^18`.
    let n = unsafe { scaled.to_int_unchecked::<i64>() };

    if n == 0 {
        // |x| < ln2/2N ≈ 0.0027.  Computing exp(x) − 1 here would lose the small
        // result in the double-double's floor relative to 1, so evaluate
        // expm1(x) = x · S(x) with S(x) = (exp(x) − 1)/x = ∑ xᵏ/(k+1)!.  S is built
        // by double-double Horner so the result keeps full *relative* accuracy.
        use exp_consts::EXPM1_S_COEFFS;

        let (high, low) = EXPM1_S_COEFFS[EXPM1_S_COEFFS.len() - 1];
        let mut s = Sum { high, low };

        for &(high, low) in EXPM1_S_COEFFS[..EXPM1_S_COEFFS.len() - 1].iter().rev() {
            s = s * x + Sum { high, low };
        }

        let result = s * x;
        return result.high + result.low;
    }

    let j = (n & (EXP_N - 1)) as usize;
    let q = n >> 7;

    let a = scaled.mul_add(-LN2_OVER_N_HI, x);
    let r = Sum::from_sum(a, scaled * -LN2_OVER_N_LO);

    // exp(x) = 2^q · mantissa; form `2^q · mantissa − 1` as a double-double.  The
    // scaling stays normal (`q ∈ [−1022, 1023]`), and the double-double subtraction
    // absorbs the cancellation that plain `exp(x) − 1` would suffer near zero.
    let (mantissa, q) = exp_mantissa(j, q, r);
    let result = mantissa * crate::exp2i(q)
        + Sum {
            high: -1.0,
            low: 0.0,
        };

    result.high + result.low
}

/// `ln(1 + r)` as a double-double for `|r| ≤ 1/256`, via `ln(1+r) = r · P(r)`.
#[inline]
fn ln_1p_kernel(r: Sum) -> Sum {
    use log_consts::LN1P_P_COEFFS;

    let (high, low) = LN1P_P_COEFFS[LN1P_P_COEFFS.len() - 1];
    let mut p = Sum { high, low };

    for &(high, low) in LN1P_P_COEFFS[..LN1P_P_COEFFS.len() - 1].iter().rev() {
        p = p * r + Sum { high, low };
    }

    p * r
}

/// Decompose a finite positive `x` into `(e, i, m·inv − 1)` for the log family.
///
/// `x = 2^e · m` with `m ∈ [1, 2)`; `i` is the 7-bit table index and the returned
/// double-double is `r = m · INV_TABLE[i] − 1 ∈ [−1/256, 1/256]`.
#[inline]
fn log_reduce(x: f64) -> (i64, usize, Sum) {
    use log_consts::INV_TABLE;

    let (_, Magnitude::Normalized(magnitude)) = normalize(x) else {
        // Callers guarantee a finite positive x.
        unreachable!()
    };

    let e = (magnitude >> EXP_SHIFT) - 1023;
    let i = ((magnitude >> (EXP_SHIFT - 7)) & 127) as usize;
    let m = f64::from_bits((magnitude as u64 & 0x000F_FFFF_FFFF_FFFF) | 0x3FF0_0000_0000_0000);

    // r = m·inv − 1 as a double-double.  `m·inv ∈ [≈0.996, 1.004]`, so the high
    // word minus one is exact (Sterbenz).
    let mi = Sum::from_product(m, INV_TABLE[i]);
    let r = kernel::fast_sum(mi.high - 1.0, mi.low);

    (e, i, r)
}

/// The natural logarithm of a finite positive `x ≠ 1`, as a double-double.
#[inline]
fn ln_dd(x: f64) -> Sum {
    use log_consts::{LN2_HI, LN2_LO, L_TABLE};

    // ln(x) = e·ln2 + L_TABLE[i] + ln(1+r).
    let (e, i, r) = log_reduce(x);
    let e = e as f64;

    let e_ln2 = Sum {
        high: e * LN2_HI,
        low: e * LN2_LO,
    };
    let (high, low) = L_TABLE[i];
    e_ln2 + Sum { high, low } + ln_1p_kernel(r)
}

/// The natural logarithm
#[must_use]
#[inline]
pub fn ln(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x == 1.0 || x == f64::INFINITY {
        return x - 1.0;
    }

    let result = ln_dd(x);
    result.high + result.low
}

/// The base-2 logarithm
#[must_use]
#[inline]
pub fn log2(x: f64) -> f64 {
    use log_consts::{LOG2_E_HI, LOG2_E_LO};

    if x.is_nan() {
        return x;
    }
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x == 1.0 || x == f64::INFINITY {
        return x - 1.0;
    }

    // log2(x) = ln(x) · log2(e)
    let result = ln_dd(x)
        * Sum {
            high: LOG2_E_HI,
            low: LOG2_E_LO,
        };
    result.high + result.low
}

/// The base-10 logarithm
#[must_use]
#[inline]
pub fn log10(x: f64) -> f64 {
    use log_consts::{LOG10_E_HI, LOG10_E_LO};

    if x.is_nan() {
        return x;
    }
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x == 1.0 || x == f64::INFINITY {
        return x - 1.0;
    }

    // log10(x) = ln(x) · log10(e)
    let result = ln_dd(x)
        * Sum {
            high: LOG10_E_HI,
            low: LOG10_E_LO,
        };
    result.high + result.low
}

/// Compute `ln(1 + x)` accurately, especially for small `x`
#[must_use]
#[inline]
pub fn ln_1p(x: f64) -> f64 {
    use log_consts::{INV_TABLE, LN2_HI, LN2_LO, L_TABLE};

    if x.is_nan() || x == 0.0 {
        // Preserve the sign of zero: ln_1p(±0) = ±0.
        return x;
    }
    if x < -1.0 {
        return f64::NAN;
    }
    if x == -1.0 {
        return f64::NEG_INFINITY;
    }
    if x == f64::INFINITY {
        return x;
    }

    // Small |x|: evaluate ln(1+x) = x·P(x) directly.  This avoids the table's
    // `L[i] + ln(1+r)` cancellation, which would cap accuracy when the result is
    // tiny, and `x` is already an exact reduced argument in the kernel's range.
    if x.abs() < 1.0 / 256.0 {
        let result = ln_1p_kernel(Sum { high: x, low: 0.0 });
        return result.high + result.low;
    }

    // Otherwise carry 1 + x exactly as `s + c` (Fast2Sum), so the bits of `x` lost
    // in forming `s` are kept in `c`.  `1 + x` is commutative, so the high word is
    // the same either way; only the residual depends on which operand is larger.
    let s = 1.0 + x;
    let c = if x.abs() <= 1.0 {
        x - (s - 1.0)
    } else {
        1.0 - (s - x)
    };

    // Reduce s = 2^e·m as for `ln`, then fold the tail `c` *exactly* into the
    // reduced argument: with δ = c·2⁻ᵉ the true mantissa is m + δ, so
    // r_full = m·inv − 1 + δ·inv reflects 1 + x = s + c with no lost bits.
    let (e, i, r) = log_reduce(s);
    let inv = INV_TABLE[i];
    let delta = if e > -1000 { c * crate::exp2i(-e) } else { 0.0 };
    let r = r + Sum::from_product(delta, inv);

    let e = e as f64;
    let e_ln2 = Sum {
        high: e * LN2_HI,
        low: e * LN2_LO,
    };
    let (high, low) = L_TABLE[i];
    let result = e_ln2 + Sum { high, low } + ln_1p_kernel(r);
    result.high + result.low
}

/// The logarithm of `x` to an arbitrary `base`
///
/// This is the ratio `log2(x) / log2(base)`; as a quotient of two logarithms it
/// is faithfully — not correctly — rounded.
#[must_use]
#[inline]
pub fn log(x: f64, base: f64) -> f64 {
    log2(x) / log2(base)
}

/// Multiply `x` by 2 raised to the power `n`
#[must_use]
#[inline]
pub const fn ldexp(x: f64, n: i32) -> f64 {
    // Scale in up to two steps per direction so the whole exponent range is
    // covered while the final multiply rounds at most once (into the subnormals).
    let mut x = x;
    let mut n = n;

    if n > 1023 {
        x *= crate::exp2i(1023);
        n -= 1023;
        if n > 1023 {
            x *= crate::exp2i(1023);
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        // 2^-969 keeps `x` normal while shedding most of a large negative `n`.
        x *= crate::exp2i(-969);
        n += 969;
        if n < -1022 {
            x *= crate::exp2i(-969);
            n += 969;
            if n < -1022 {
                n = -1022;
            }
        }
    }

    x * f64::from_bits(((0x3ff + n) as u64) << EXP_SHIFT)
}

/// Decompose into a significand and an exponent
///
/// The absolute value of the significand is in the range of [0.5, 1) for
/// nonzero finite `x` for historical reasons.  This function also explains how
/// [`f64::MAX_EXP`] and [`f64::MIN_EXP`] are defined.
#[must_use]
#[inline]
pub const fn frexp(x: f64) -> (f64, i32) {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return (x, 0);
    };

    let mask = f64::MIN_POSITIVE.to_bits() - 1;
    let significand = magnitude as u64 & mask | 0.5f64.to_bits();

    #[allow(clippy::cast_possible_truncation)]
    (
        f64::from_bits(crate::u64_sign_bit(sign) | significand),
        f64::MIN_EXP - 1 + (magnitude >> EXP_SHIFT) as i32,
    )
}

/// `eˣ` and `e⁻ˣ` combined as `2`<sup>`q−1`</sup>` · (m ± 2⁻²q/m)` for `x ≥ 0`.
///
/// `(m, q)` is `eˣ = 2`<sup>`q`</sup>` · m`.  The returned double-double is the
/// mantissa `m ± t` (t = e⁻²ˣ-scaled reciprocal); the caller scales by `2`<sup>`q−1`</sup>.
#[inline]
fn cosh_sinh_mantissa(x: f64, add: bool) -> (Sum, i64) {
    let (m, q) = exp_dd(x);

    // t = 2⁻²q / m ≈ e⁻ˣ relative to eˣ.  `exp2i(-2q)` underflows to 0 once the
    // term is negligible, so no explicit cutoff is needed.
    let t = m.recip() * crate::exp2i(-2 * q);
    let mantissa = if add {
        m + t
    } else {
        m + Sum {
            high: -t.high,
            low: -t.low,
        }
    };
    (mantissa, q)
}

/// Hyperbolic cosine
#[must_use]
#[inline]
pub fn cosh(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }

    let x = x.abs();

    // `ln(2·f64::MAX)`: above this `cosh = eˣ/2` overflows.
    if x > 710.475_860_073_944 {
        return f64::INFINITY;
    }

    // cosh(x) = ½(eˣ + e⁻ˣ) = 2^(q−1)·(m + 2⁻²q/m); the sum never cancels.
    let (mantissa, q) = cosh_sinh_mantissa(x, true);
    kernel::fast_ldexp(mantissa.high + mantissa.low, q - 1)
}

/// Hyperbolic sine
#[must_use]
#[inline]
pub fn sinh(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }

    let s = x.abs();

    if s > 710.475_860_073_944 {
        return f64::INFINITY.copysign(x);
    }

    // For |x| ≤ 2⁻²⁶, sinh(x) = x + x³/6 + … rounds to exactly x.
    if s < 1.490_116_119_384_765_6e-8 {
        return x;
    }

    // sinh(x) = ½(eˣ − e⁻ˣ) = 2^(q−1)·(m − 2⁻²q/m).  The double-double subtraction
    // `m − 2⁻²q/m` captures the cancellation exactly (2Sum), so no separate
    // small-argument polynomial is needed above the 2⁻²⁶ threshold.
    let (mantissa, q) = cosh_sinh_mantissa(s, false);
    kernel::fast_ldexp(mantissa.high + mantissa.low, q - 1).copysign(x)
}

/// Hyperbolic tangent
#[must_use]
#[inline]
pub fn tanh(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }

    let s = x.abs();

    // For x ≳ 19, tanh(x) = 1 − 2e⁻²ˣ rounds to exactly 1 (and this keeps e²ˣ from
    // overflowing below).
    if s >= 20.0 {
        return 1.0_f64.copysign(x);
    }

    // For |x| ≤ 2⁻²⁷, tanh(x) = x − x³/3 + … rounds to exactly x.  (The cubic term
    // is twice sinh's, so the threshold is half a binade smaller.)
    if s < 7.450_580_596_923_828e-9 {
        return x;
    }

    // tanh(x) = expm1(2x) / (expm1(2x) + 2).  Form expm1(2x) = 2^q·m − 1 as a
    // double-double, then the quotient in double-double.
    let (m, q) = exp_dd(2.0 * s);
    let t = m * crate::exp2i(q)
        + Sum {
            high: -1.0,
            low: 0.0,
        };
    let result = t
        * (t + Sum {
            high: 2.0,
            low: 0.0,
        })
        .recip();
    (result.high + result.low).copysign(x)
}
