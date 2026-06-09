use crate::Sign;
use core::cmp::Ordering;
use core::f32;
use core::num::FpCategory;

/// Rounds half-way cases away from zero
#[must_use]
#[inline]
pub fn roundf(x: f32) -> f32 {
    let r = x.abs();
    let i = r.trunc();

    (i + f32::from(r - i >= 0.5)).copysign(x)
}

/// The cube root
#[must_use]
#[inline]
pub fn cbrtf(x: f32) -> f32 {
    let (sign, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return x;
    };

    let magnitude = (0x2A51_2CE3 + magnitude / 3) as u32;
    let x: f64 = x.into();
    let y: f64 = f32::from_bits(u32_sign_bit(sign) | magnitude).into();
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
pub fn hypotf(x: f32, y: f32) -> f32 {
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
        return crate::fma(
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
    if crate::fma(candidate_f64, candidate_f64, -big2) - small2 == 0.0 {
        return candidate;
    }

    // One-step Newton correction in f64, then round-to-odd via the sign of the
    // low part of the (r_hi, r_lo) double-double.
    let residual = (big2 - r2) + small2 - crate::fma(r, r, -r2);
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

/// Higher part of ln(2) whose lowest 14 bits are zero
pub const LN_2_HI: f64 = 0.693_147_180_560_117_7;

/// Lower part of ln(2)
///
/// To be precise, this is the `f64` closest to ln(2) - [`LN_2_HI`].
pub const LN_2_LO: f64 = -1.723_944_452_561_483_5e-13;

const _: () = assert!(LN_2_HI + LN_2_LO == core::f64::consts::LN_2);

/// Explicitly stored significand bits in [`prim@f32`]
///
/// This constant is usually used as a shift to access the exponent bits.
pub const EXP_SHIFT: u32 = f32::MANTISSA_DIGITS - 1;

/// Magnitude of `f32`
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
    /// The layout of the bits is the same as a normal positive `f32`.  For
    /// subnormal numbers, the stored exponent becomes zero or negative while
    /// the significand is normalized to have an implicit leading bit.
    Normalized(i32),
}

/// Break a `f32` into its sign and magnitude
#[inline]
pub const fn normalize(x: f32) -> (Sign, Magnitude) {
    let sign = if x.is_sign_negative() {
        Sign::Negative
    } else {
        Sign::Positive
    };
    let magnitude = x.abs().to_bits() as i32;

    match x.classify() {
        FpCategory::Nan => (sign, Magnitude::Nan),
        FpCategory::Infinite => (sign, Magnitude::Infinite),
        FpCategory::Zero => (sign, Magnitude::Zero),
        FpCategory::Normal => (sign, Magnitude::Normalized(magnitude)),
        FpCategory::Subnormal => {
            const EXPONENT_DIGITS: u32 = 32 - f32::MANTISSA_DIGITS;
            let shift = magnitude.leading_zeros() as i32 - EXPONENT_DIGITS as i32;
            let magnitude = (magnitude << shift) - (shift << EXP_SHIFT);
            (sign, Magnitude::Normalized(magnitude))
        }
    }
}

/// Sign bit of an `f32`, placed at bit 31
pub const fn u32_sign_bit(sign: Sign) -> u32 {
    match sign {
        Sign::Positive => 0,
        Sign::Negative => 1 << 31,
    }
}

/// Correctly-rounded fused multiply-add (f32)
///
/// Always computes `x * y + a` as a single fused operation.  On `x86`/`x86_64`
/// without a compile-time `+fma` target feature the FMA instruction is
/// selected at runtime; on targets where the FMA instruction is unavailable
/// it falls back to the platform's software `fmaf` implementation.
///
/// Use this instead of `x.mul_add(y, a)` for error-free transforms and
/// residual tests on `f32` values.  For polynomial hot paths where a lost low
/// bit is acceptable, prefer `fast_mul_add` (via `f64` promotion).
// Not `const`: the hardware path calls the non-const `f32::mul_add`.
#[must_use]
#[allow(
    unreachable_code,
    clippy::missing_const_for_fn,
    clippy::disallowed_methods
)]
#[inline]
pub fn fmaf(x: f32, y: f32, a: f32) -> f32 {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[cfg(not(target_feature = "fma"))]
    {
        #[target_feature(enable = "fma")]
        unsafe fn force_fma(x: f32, y: f32, a: f32) -> f32 {
            x.mul_add(y, a)
        }

        if std::is_x86_feature_detected!("fma") {
            // SAFETY: runtime check confirmed FMA is available on this CPU.
            return unsafe { force_fma(x, y, a) };
        }
    }

    x.mul_add(y, a)
}
