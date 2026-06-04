use super::{normalize, Magnitude};
use core::cmp::Ordering;
use core::f32;

/// Rounds half-way cases away from zero
#[must_use]
#[inline]
pub fn round(x: f32) -> f32 {
    let r = x.abs();
    let i = r.trunc();

    (i + f32::from(r - i >= 0.5)).copysign(x)
}

/// The cube root
#[must_use]
#[inline]
pub fn cbrt(x: f32) -> f32 {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return x;
    };

    let magnitude = (0x2A51_2CE3 + magnitude / 3) as u32;
    let x: f64 = x.into();
    let y: f64 = f32::from_bits(crate::u32_sign_bit(sign) | magnitude).into();
    let y = y * (0.5 + 1.5 * x / crate::fast_mul_add(2.0 * y, y * y, x));
    let y = y * (0.5 + 1.5 * x / crate::fast_mul_add(2.0 * y, y * y, x));

    y as f32
}

/// Hypotenuse of a right-angled triangle with sides `x` and `y`
///
/// Mirrors CORE-MATH's hot path: sort to `at ≥ ay`, compute `r² = at² + ay²`
/// and `r = √r²` in f64, then dispatch on cheap integer-bit tests.  The
/// degenerate `ay/at < 2⁻¹³` case skips the square root entirely; the bulk of
/// `r` values are far from an f32 midpoint and return the single rounding `c =
/// r as f32`; only the tight-rounding sliver runs the residual correction.
#[must_use]
#[inline]
pub fn hypot(x: f32, y: f32) -> f32 {
    /// Largest f64 ≤ 2⁻¹³ (= `0x1.fffffep-13`); the small-`ay` cutoff
    const SMALL_RATIO: f64 = f64::from_bits(0x3F2F_FFFF_C000_0000);

    /// 2⁻¹³ as f32; the FMA coefficient on the small-`ay` fast path
    const HALF_ULP_F32: f32 = f32::from_bits(0x3900_0000);

    /// `f32::MAX` cast to f64 (`0x1.fffffep+127`); above this, `r as f32` overflows
    const F32_MAX_AS_F64: u64 = 0x47EF_FFFF_E000_0000;

    /// Width of the f64 → f32 rounding window: 28 = 52 − 24 low bits
    const TAIL_MASK: u64 = 0x0FFF_FFFF;

    let ax = x.abs();
    let ay = y.abs();

    if ax == f32::INFINITY || ay == f32::INFINITY {
        return f32::INFINITY;
    }
    if !ax.is_finite() || !ay.is_finite() {
        return ax + ay; // propagates NaN
    }

    let big_f32 = ax.max(ay);
    let small_f32 = ax.min(ay);
    let big = f64::from(big_f32);
    let small = f64::from(small_f32);
    let big2 = big * big;
    let small2 = small * small;

    // `small ≤ 2⁻¹³·big` ⇒ hypot ≈ big + small²/(2·big), and `big + 2⁻¹³·small`
    // via a single FMA correctly rounds throughout that range without ever
    // taking a sqrt.
    if small < big * SMALL_RATIO {
        return crate::correct_mul_add(
            f64::from(HALF_ULP_F32),
            f64::from(small_f32),
            f64::from(big_f32),
        ) as f32;
    }

    let r2 = big2 + small2;
    let r = r2.sqrt();
    let candidate = r as f32;

    // Past f32::MAX, the single rounding `r as f32` already produces the
    // correct overflow (MAX or +∞); the midpoint gate below would read
    // meaningless low bits.
    if r.to_bits() > F32_MAX_AS_F64 {
        return candidate;
    }

    // Common case: `r`'s low f64 tail is far from an f32 midpoint, so the
    // single rounding is correct.
    if (r.to_bits().wrapping_add(1) & TAIL_MASK) > 2 {
        return candidate;
    }

    // Residual test: if `candidate² == big² + small²` exactly, we're done.
    let candidate_f64 = f64::from(candidate);
    if crate::correct_mul_add(candidate_f64, candidate_f64, -big2) - small2 == 0.0 {
        return candidate;
    }

    // One-step Newton correction in f64, then round-to-odd via the sign of the
    // low part of the (r_hi, r_lo) double-double.
    let residual = (big2 - r2) + small2 - crate::correct_mul_add(r, r, -r2);
    let correction = r * (0.5 / r2) * residual;
    let r_hi = r + correction;
    let r_lo = correction + (r - r_hi);
    let bits = r_hi.to_bits();

    let bits = match (bits & TAIL_MASK, r_lo.partial_cmp(&0.0)) {
        (0, Some(Ordering::Greater)) => bits.wrapping_add(1),
        (0, Some(Ordering::Less)) => bits.wrapping_sub(1),
        _ => bits,
    };
    f64::from_bits(bits) as f32
}
