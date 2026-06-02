use super::double::{fast_sum, round_general64, Sum};
use super::{normalize, Magnitude, EXP_SHIFT};

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
    let sum = fast_sum(2.0 * y, quotient.high);
    let sum = Sum {
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
    const SMALL_RATIO: f64 = 7.450580596923828e-9; // 2⁻²⁷
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
    let s2 = Sum::from_product(big_s, big_s) + Sum::from_product(small_s, small_s);
    let h = s2.high.sqrt();
    let h2 = Sum::from_product(h, h);
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
    round_general64(value, i64::from(exponent))
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
