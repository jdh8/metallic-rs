use super::double::{DoubleDouble, fast_sum, round_general64};
use crate::Sign;
use core::num::FpCategory;

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
    let y = f64::from_bits(u64_sign_bit(sign) | magnitude);
    let y = crate::fast_mul_add(1.0 / 3.0, x / (y * y) - y, y);
    let y = crate::fast_mul_add(1.0 / 3.0, x / (y * y) - y, y);
    let y = y * (0.5 + 1.5 * x / crate::fast_mul_add(2.0 * y, y * y, x));

    let quotient = DoubleDouble::from_quotient(x, y) / y;
    let sum = fast_sum(2.0 * y, quotient.high);
    let sum = DoubleDouble {
        high: sum.high,
        low: quotient.low + sum.low,
    } / 3.0;

    coefficient * (sum.high + sum.low)
}

/// Hypotenuse of a right-angled triangle with sides `x` and `y`
///
/// Computes `√(x² + y²)` correctly rounded.  The larger leg is scaled into
/// `[1, 2)` (exact, scale-invariant) so neither square over- nor underflows;
/// `big² + small²` is then accumulated as a double-double, its square root is
/// refined by one Newton step to ≈2⁻¹⁰⁵, and the subnormal-safe rounder finishes.
#[must_use]
#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    let ax = x.abs();
    let ay = y.abs();

    // ±∞ in either argument ⇒ +∞, even when the other is NaN (IEEE 754 hypot).
    if ax == f64::INFINITY || ay == f64::INFINITY {
        return f64::INFINITY;
    }
    // Any remaining non-finite is NaN; `ax + ay` propagates it.
    if ax.is_nan() || ay.is_nan() {
        return ax + ay;
    }

    let big = ax.max(ay);
    let small = ax.min(ay);

    // `small == 0` (and possibly `big == 0`) ⇒ the result is exactly `big`.
    if small == 0.0 {
        return big;
    }

    // Negligible smaller leg: when `small < big·2⁻²⁷` the correction
    // `small²/(2·big) < ½ ulp(big)`, so `√(big² + small²)` rounds to `big`.  This
    // skips the square root for the common case of disparate magnitudes.
    const SMALL_RATIO: f64 = 7.450_580_596_923_828e-9; // 2⁻²⁷
    if small < big * SMALL_RATIO {
        return big;
    }

    // Overflow guard.  When `big` sits in the top binade the result can exceed
    // `f64::MAX`; rounding it through the bit-level reconstruction below would
    // lose the `MAX`-vs-`∞` distinction.  Scale both legs down by 4 (exact),
    // recurse once (the scaled `big ≤ MAX/8` cannot re-enter here), and scale the
    // result back up with `ldexp`, which saturates to `+∞` correctly and, being an
    // exact power of two, preserves correct rounding.
    if big > f64::MAX * 0.5 {
        return ldexp(hypot(big * 0.25, small * 0.25), 2);
    }

    // Scale so the larger leg lands in [1, 2): the result is then
    // `√(big_s² + small_s²) · 2ᵉ` where `e` is the unbiased exponent of `big`.
    // When `big` is normal (the overwhelmingly common case) the scale is just an
    // exponent rewrite; only a subnormal `big` (both legs subnormal) needs the
    // `frexp`/`ldexp` normalization.  `small_s` always goes through `ldexp`, which
    // stays correct when `small ≪ big` underflows it toward zero (whereupon the
    // double-double simply returns `big`).
    let bits = big.to_bits();
    let (big_s, small_s, e) = if bits >= f64::MIN_POSITIVE.to_bits() {
        let e = (bits >> EXP_SHIFT) as i32 - (f64::MAX_EXP - 1);
        let big_s = f64::from_bits(bits & (f64::MIN_POSITIVE.to_bits() - 1) | (0x3ff << EXP_SHIFT));
        (big_s, ldexp(small, -e), e)
    } else {
        let (_, n) = frexp(big);
        (ldexp(big, 1 - n), ldexp(small, 1 - n), n - 1)
    };

    // big_s² + small_s² in double-double (each square exact via the FMA in
    // `from_product`), then one Newton step `h + (s2 − h²)/(2h)` on `h = √s2.high`.
    let s2 =
        DoubleDouble::from_product(big_s, big_s) + DoubleDouble::from_product(small_s, small_s);
    let h = s2.high.sqrt();
    let h2 = DoubleDouble::from_product(h, h);
    let residual = (s2.high - h2.high) + (s2.low - h2.low);
    let corrected = fast_sum(h, residual * (0.5 / h));

    // `corrected ∈ [1, 2√2)`; normalize into [1, 2) for the subnormal-safe rounder,
    // folding the binade into the exponent `e`.  `big ≤ MAX/2` here, so the result
    // is finite and the rounder never sees overflow.
    let (value, exponent) = if corrected.high >= 2.0 {
        (corrected * 0.5, e + 1)
    } else {
        (corrected, e)
    };

    // Ziv gate: the double-double sqrt is good to ≈2⁻¹⁰⁴; if its `±eps` interval
    // does not straddle a rounding boundary, round it directly.  The few inputs
    // that straddle — CORE-MATH's hard-to-round corpus — are resolved exactly by
    // the integer [`hypot_hard`].  That exact resolver rounds to 53 bits assuming
    // a normal result, so restrict it to a normal `big` (where the result, being
    // ≥ `big`, is itself normal); a subnormal `big` keeps the dd round.
    let eps = value.high * HYPOT_ZIV_EPS;
    let lo = value.high + (value.low - eps);
    let hi = value.high + (value.low + eps);
    if lo != hi && bits >= f64::MIN_POSITIVE.to_bits() {
        return ldexp(hypot_hard(big_s, small_s), e);
    }
    round_general64(value, i64::from(exponent))
}

/// Relative half-width of the `hypot` Ziv gate; the double-double sqrt is good to
/// ≈2⁻¹⁰⁴, so `2⁻⁹⁹` keeps a comfortable margin while deferring only genuine
/// near-ties to the exact resolver.
const HYPOT_ZIV_EPS: f64 = 1.577_721_810_442_023_6e-30; // 2⁻⁹⁹

/// Exact integer resolution of the `hypot` inputs the double-double sqrt cannot
/// correctly round (≈2⁻¹⁰⁴ leaves them on the wrong side of a midpoint).
///
/// Port of CORE-MATH's `as_hypot_hard` (round-to-nearest only).  With both legs
/// scaled so `big_s ∈ [1, 2)`, it forms `m2 = (bm·2ᴮˢ)² + (lm·2ˡˢ)²` exactly as a
/// 128-bit integer (a sticky bit absorbs the smaller leg's shifted-out tail),
/// integer-searches the sqrt mantissa from just below the `f64` estimate, then
/// breaks the midpoint tie to even.  Returns √(big_s²+small_s²) ∈ [1, 2√2).
fn hypot_hard(big_s: f64, small_s: f64) -> f64 {
    let xi = big_s.to_bits();
    let yi = small_s.to_bits();
    let bm = (xi & (!0u64 >> 12)) | (1u64 << 52);
    let lm = (yi & (!0u64 >> 12)) | (1u64 << 52);
    let be = (xi >> EXP_SHIFT) as i64;
    let le = (yi >> EXP_SHIFT) as i64;

    const BS: i64 = 2;
    let ri = (big_s * big_s + small_s * small_s).sqrt().to_bits();
    let mut rm = (ri & (!0u64 >> 12)) | (1u64 << 52);
    let mut re = (ri >> EXP_SHIFT) as i64 - 0x3ff;
    // rm -= 3, borrowing across the binade so the search starts safely below √.
    for _ in 0..3 {
        if rm == 1u64 << 52 {
            rm = !0u64 >> 11;
            re -= 1;
        } else {
            rm -= 1;
        }
    }

    let bm = bm << BS;
    let mut m2 = u128::from(bm) * u128::from(bm);
    let ls = BS - (be - le);
    if ls >= 0 {
        let lm = lm << ls;
        m2 += u128::from(lm) * u128::from(lm);
    } else {
        let lm2 = u128::from(lm) * u128::from(lm);
        let shift = (-ls * 2) as u32;
        m2 += lm2 >> shift;
        if lm2 << (128 - shift) != 0 {
            m2 |= 1; // sticky bit for the shifted-out tail
        }
    }

    let k = (BS + re) as u32;
    let mut d: i128;
    loop {
        rm += 1 + u64::from(rm >= (1u64 << 53));
        let tm = rm << k;
        d = m2 as i128 - (u128::from(tm) * u128::from(tm)) as i128;
        if d <= 0 {
            break;
        }
    }
    if d != 0 {
        // `rm² > m2`: compare `m2` against the midpoint half a step below `rm`.
        let half = 1u64 << (k - u32::from(rm <= (1u64 << 53)));
        let tm = (rm << k) - half;
        let dmid = m2 as i128 - (u128::from(tm) * u128::from(tm)) as i128;
        if dmid != 0 {
            rm = (rm as i64 + (dmid >> 127) as i64) as u64; // round down if below midpoint
        } else {
            rm -= rm & 1; // exact midpoint → round to even
        }
    }
    if rm >= (1u64 << 53) {
        rm >>= 1;
        re += 1;
    }

    // big_s ∈ [1, 2) gives biased exponent 0x3ff; `rm ∈ [2⁵², 2⁵³)`, so its
    // implicit bit carries into the exponent field: result = ((0x3fe+re)<<52) + rm.
    f64::from_bits((((0x3fe + re) as u64) << EXP_SHIFT) + rm)
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
        f64::from_bits(u64_sign_bit(sign) | significand),
        f64::MIN_EXP - 1 + (magnitude >> EXP_SHIFT) as i32,
    )
}

/// Explicitly stored significand bits in [`prim@f64`]
///
/// This constant is usually used as a shift to access the exponent bits.
pub const EXP_SHIFT: u32 = f64::MANTISSA_DIGITS - 1;

/// Magnitude of `f64`
///
/// Nonzero subnormal numbers are normalized to have an implicit leading bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Magnitude {
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
pub const fn normalize(x: f64) -> (Sign, Magnitude) {
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

/// Sign bit of an `f64`, placed at bit 63
const fn u64_sign_bit(sign: Sign) -> u64 {
    match sign {
        Sign::Positive => 0,
        Sign::Negative => 1 << 63,
    }
}

/// Fast multiply-add
///
/// This function picks the faster way to compute `x * y + a` depending on the
/// target architecture.  The FMA instruction is used if available.  Otherwise,
/// it falls back to `x * y + a` that is faster but gives less accurate results
/// than a true FMA.
///
/// # Not an error-free transform
///
/// Because the fallback path rounds the product *before* the addition, this
/// helper is **not** fused on every target.  Do not use it where correctness
/// depends on the single rounding of a true FMA — error-free transforms,
/// residual tests, and high-precision compensation must use
/// [`fma`].  Reserve this for hot polynomial-style spots where a lost low bit
/// is absorbed by later rounding.
// Not `const`: the hardware path calls the non-const `f64::mul_add`.
#[allow(
    unreachable_code,
    clippy::missing_const_for_fn,
    clippy::disallowed_methods
)]
#[inline]
pub fn fast_mul_add(x: f64, y: f64, a: f64) -> f64 {
    #[cfg(feature = "_no_fma")]
    #[allow(clippy::suboptimal_flops)]
    return x * y + a;

    // x86/x86_64 without compile-time FMA: runtime dispatch.
    // `is_x86_feature_detected!` caches via an AtomicU8 (one-time CPUID cost),
    // so subsequent calls pay only an atomic load plus a branch the predictor
    // always gets right.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[cfg(not(target_feature = "fma"))]
    {
        #[target_feature(enable = "fma")]
        unsafe fn force_fma(x: f64, y: f64, a: f64) -> f64 {
            x.mul_add(y, a)
        }

        if std::is_x86_feature_detected!("fma") {
            // SAFETY: runtime check confirmed FMA is available on this CPU.
            return unsafe { force_fma(x, y, a) };
        }

        #[allow(clippy::suboptimal_flops)]
        return x * y + a;
    }

    // Every other target (compile-time FMA, aarch64 where fp-armv8 is
    // baseline, wasm32, …): delegate to Rust's `mul_add`, which LLVM
    // lowers correctly.
    x.mul_add(y, a)
}

/// Correctly-rounded fused multiply-add (f64)
///
/// Always computes `x * y + a` as a single fused operation.  On `x86`/`x86_64`
/// without a compile-time `+fma` target feature the FMA instruction is
/// selected at runtime; on targets where the FMA instruction is unavailable
/// it falls back to the platform's software `fma` implementation.
///
/// Use this instead of `x.mul_add(y, a)` for error-free transforms, residual
/// tests, and double-double compensation, where the single rounding of a true
/// FMA is required for correctness.  For polynomial hot paths where a lost low
/// bit is acceptable, prefer `fast_mul_add`, which avoids the software `fma`
/// fallback cost on old hardware.
// Not `const`: the hardware path calls the non-const `f64::mul_add`.
#[must_use]
#[allow(
    unreachable_code,
    clippy::missing_const_for_fn,
    clippy::disallowed_methods
)]
#[inline]
pub fn fma(x: f64, y: f64, a: f64) -> f64 {
    // x86/x86_64 without compile-time FMA: runtime dispatch to the hardware
    // FMA instruction; fall back to the software `fma` for old CPUs.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[cfg(not(target_feature = "fma"))]
    {
        #[target_feature(enable = "fma")]
        unsafe fn force_fma(x: f64, y: f64, a: f64) -> f64 {
            x.mul_add(y, a)
        }

        if std::is_x86_feature_detected!("fma") {
            // SAFETY: runtime check confirmed FMA is available on this CPU.
            return unsafe { force_fma(x, y, a) };
        }
    }

    x.mul_add(y, a)
}

/// Const evaluation of 2<sup>`n`</sup>
#[inline]
pub const fn exp2i(n: i64) -> f64 {
    let bits = match n + 1023 {
        2047.. => return f64::INFINITY,
        s @ 1..=2046 => s << EXP_SHIFT,
        s @ -63..=0 => 1 << (EXP_SHIFT - 1) >> -s,
        _ => 0,
    };
    #[allow(clippy::cast_sign_loss)]
    f64::from_bits(bits as u64)
}

#[allow(clippy::float_cmp)]
const _: () = {
    let (mut n, mut x) = (0, 1.0);

    while n < 1100 {
        assert!(exp2i(n) == x);
        x *= 2.0;
        n += 1;
    }

    (n, x) = (0, 1.0);

    while n > -1100 {
        assert!(exp2i(n) == x);
        x *= 0.5;
        n -= 1;
    }
};
