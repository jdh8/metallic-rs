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

    // Seed the *inverse* cube root `c ≈ x^(−1/3)` by the magic-constant bit trick
    // (the exponent is negated relative to a cube-root seed), then refine it with
    // no division at all — where the old forward Newton/Halley chain spent three.
    // The residual `h = 1 − x·c³` drives a degree-4 correction `c·(1 − h)^(−1/3)`
    // (the series `1 + h/3 + 2h²/9 + 14h³/81 + 35h⁴/243`, exact through `h⁴`),
    // then one cubic Newton step `c·(1 + h/3 + 2h²/9)` lands `c` at f64 accuracy.
    let magnitude = (0x553E_F100_0000_0000 - magnitude / 3) as u64;
    let c = f64::from_bits(u64_sign_bit(sign) | magnitude);
    let h = crate::fma(-x, c * c * c, 1.0);
    let c = c * crate::poly(h, &[1.0, 1.0 / 3.0, 2.0 / 9.0, 14.0 / 81.0, 35.0 / 243.0]);
    let h = crate::fma(-x, c * c * c, 1.0);
    let c = c * crate::fast_mul_add(h, crate::fast_mul_add(2.0 / 9.0, h, 1.0 / 3.0), 1.0);

    // Forward result `y = x·c² ≈ x^(1/3)`, then one Newton step in plain `f64` —
    // the division-free fast leg.  `1/y² = c²` exactly in the limit (`y·c = 1`),
    // so the Newton correction is `dy = −(y³ − x)·c²/3` with no division; `y³ − x`
    // is formed with an exact FMA so its leading cancellation survives.  The
    // result rides as a double-double `(high, low) = fast_sum(y, dy)`; the Ziv
    // gate accepts it unless it straddles a rounding boundary, where the accurate
    // double-double leg takes over.  This keeps the costly `from_quotient`/`/y`
    // divisions off the common path entirely.
    let c2 = c * c;
    let y = x * c2;
    let y2 = DoubleDouble::from_product(y, y);
    let resid = crate::fma(y, y2.high, -x) + y * y2.low; // y³ − x
    let dy = (-1.0 / 3.0 * c2) * resid;
    let DoubleDouble { high, low } = fast_sum(y, dy);
    let err = CBRT_ZIV_EPS * high.abs();
    let lo = high + (low - err);
    let hi = high + (low + err);
    if lo == hi {
        return coefficient * lo;
    }

    // Accurate leg: the correctly-rounded double-double Newton finish.  First
    // polish `y` to ~1 ulp with one more division-free Newton step, so this rare
    // last-resort path starts from a tighter seed than the gated leg's ~3-ulp `y`.
    let y = crate::fast_mul_add(-(1.0 / 3.0) * c2, crate::fma(y, y * y, -x), y);
    let quotient = DoubleDouble::from_quotient(x, y) / y;
    let sum = fast_sum(2.0 * y, quotient.high);
    let sum = DoubleDouble {
        high: sum.high,
        low: quotient.low + sum.low,
    } / 3.0;

    coefficient * (sum.high + sum.low)
}

/// Ziv gate for `cbrt`'s division-free fast leg, as a *relative* bound on the
/// result.  The fast leg is one plain-`f64` Newton step from a ~3-ulp seed, so
/// its error rides the Newton residual `≈(3 ulp)² ≈ 2⁻¹⁰⁰` plus the `f64`
/// rounding of `dy`; `2⁻⁹⁵` keeps a >20× margin (certified by
/// `ziv_soundness::cbrt_fast_leg_is_sound`).  On a straddle the accurate
/// double-double leg rounds correctly.
const CBRT_ZIV_EPS: f64 = 2.524_354_896_707_238_4e-29; // 2⁻⁹⁵

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
    // A plain `f64` sqrt estimate to seed the integer search — not `hypot`, whose
    // extra accuracy is exactly what the integer refinement supplies.
    #[allow(clippy::imprecise_flops)]
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

/// MPFR-certified soundness of `cbrt`'s fast-leg Ziv gate.  The gate may only
/// confirm a rounding when its half-width [`CBRT_ZIV_EPS`] truly exceeds the
/// fast leg's error; otherwise a confident `lo == hi` could certify a value on
/// the wrong side of a boundary.  Run with `--features mpfr`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::*;
    use rug::Float;

    fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Replicates [`cbrt`]'s positive-`x` division-free fast leg (seed →
    /// refinement → one Newton step), returning the gated double-double.
    fn cbrt_fast_dd(x: f64) -> DoubleDouble {
        let magnitude = (0x553E_F100_0000_0000 - (x.to_bits() as i64) / 3) as u64;
        let c = f64::from_bits(magnitude);
        let h = crate::fma(-x, c * c * c, 1.0);
        let c = c * crate::poly(h, &[1.0, 1.0 / 3.0, 2.0 / 9.0, 14.0 / 81.0, 35.0 / 243.0]);
        let h = crate::fma(-x, c * c * c, 1.0);
        let c = c * crate::fast_mul_add(h, crate::fast_mul_add(2.0 / 9.0, h, 1.0 / 3.0), 1.0);
        let c2 = c * c;
        let y = x * c2;
        let y2 = DoubleDouble::from_product(y, y);
        let resid = crate::fma(y, y2.high, -x) + y * y2.low;
        let dy = (-1.0 / 3.0 * c2) * resid;
        fast_sum(y, dy)
    }

    /// Worst `|leg(x) − cbrt(x)| / (CBRT_ZIV_EPS · |result|)` over `[0.5, 4)` —
    /// one full period of the magic-constant seed (every mantissa × exponent
    /// mod 3, the only inputs the relative error depends on).  A ratio `< 0.5`
    /// certifies the 2× soundness margin.
    #[test]
    fn cbrt_fast_leg_is_sound() {
        let (lb, hb) = (0.5f64.to_bits(), 4.0f64.to_bits());
        let mut worst = 0.0f64;
        let mut worst_x = 0.5;
        for i in 0..30_000_000u64 {
            let x = f64::from_bits(lb + mix(i) % (hb - lb));
            let e = cbrt_fast_dd(x);
            let got = Float::with_val(250, e.high) + Float::with_val(250, e.low);
            let truth = Float::with_val(250, x).cbrt();
            let abs = Float::with_val(250, &got - &truth).abs().to_f64();
            let ratio = abs / (CBRT_ZIV_EPS * e.high.abs());
            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        println!("cbrt fast leg: worst |err|/gate = {worst:.4} at x={worst_x:e}");
        assert!(
            worst < 0.5,
            "cbrt gate covers only {:.2}× the slip at x={worst_x:e}",
            1.0 / worst
        );
    }
}
