use super::cbrt_tables::{NEWTON, SEED_CURVE, SEED_SLOPE, SEED_VALUE};
use super::uint::{cmp_384, mhi_approx, mul_hi_64, shl_384, wmul};
use super::{
    BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, MANTISSA_MASK, Magnitude, QUIET_BIT, SIGN_MASK,
    fma128, normalize, split,
};
use core::cmp::Ordering;

/// The square root.
#[must_use]
#[inline]
pub fn sqrtq(x: f128) -> f128 {
    if x.is_nan() {
        return f128::from_bits(x.to_bits() | QUIET_BIT);
    }
    x.sqrt()
}

/// The reciprocal square root.
#[must_use]
#[inline]
pub fn rsqrtq(x: f128) -> f128 {
    let bits = x.to_bits();
    let magnitude = bits & !SIGN_MASK;

    if magnitude > EXP_MASK {
        return f128::from_bits(bits | QUIET_BIT);
    }
    if magnitude == 0 {
        return f128::from_bits((bits & SIGN_MASK) | EXP_MASK);
    }
    if bits & SIGN_MASK != 0 {
        return f128::NAN;
    }
    if magnitude == EXP_MASK {
        return 0.0;
    }

    let (_, Magnitude::Normalized(magnitude)) = normalize(x) else {
        unreachable!()
    };
    let (mantissa, exponent) = parts(magnitude);

    // The native square root and division give a seed within a few ulps.  One
    // compensated Newton step makes the exact midpoint walk below normally a
    // no-op, while that walk remains the final authority on rounding.
    let r = 1.0 / x.sqrt();
    let rx = r * x;
    let drx = fma128(r, x, -rx);
    let h = fma128(r, rx, -1.0) + r * drx;
    let candidate = fma128(-(r * 0.5), h, r);

    correct_rsqrt(mantissa, exponent, candidate)
}

/// Half-width, in units of the candidate's 2^-123, of the rounding-tie window
/// [`cbrtq`] refuses to decide from the fast candidate and hands to the exact
/// midpoint walk instead.  [`cbrt_fixed`] is analytically within a dozen units
/// of the true cube root; [`ziv_soundness`] certifies the ≥ 2× margin the
/// project requires.
const CBRT_GATE: u128 = 32;

/// The cube root.
#[must_use]
#[inline]
pub fn cbrtq(x: f128) -> f128 {
    let bits = x.to_bits();
    let magnitude = bits & !SIGN_MASK;

    // ±0, ±∞, and NaN return themselves (NaN quietened) — bit tests only, no
    // soft-float classification.
    if magnitude == 0 || magnitude >= EXP_MASK {
        return if magnitude > EXP_MASK {
            f128::from_bits(bits | QUIET_BIT)
        } else {
            x
        };
    }
    let (mantissa, exponent) = split(magnitude);
    let remainder = exponent.rem_euclid(3);
    let scale = exponent.div_euclid(3);

    let candidate = cbrt_fixed(mantissa, remainder);
    // Round the Q123 candidate to 113 bits.  Composing the bits by adding the
    // significand onto a BIAS − 1 exponent field lets a carry out of the
    // rounding land on an exact 2 instead of overflowing the mantissa.
    let rounded = f128::from_bits((((BIAS - 1) as u128) << EXP_SHIFT) + ((candidate + 1024) >> 11));
    let rest = candidate & 2047;
    let result = if rest.abs_diff(1024) <= CBRT_GATE {
        correct_cbrt(mantissa, remainder, rounded)
    } else {
        rounded
    };

    // A finite cube root lands thousands of binades inside the normal range,
    // so the 2^scale factor is a plain exponent-field addition — no soft-float
    // `ldexp` multiply, no subnormal concerns.
    let scaled = result
        .to_bits()
        .wrapping_add_signed(i128::from(scale) << EXP_SHIFT);
    f128::from_bits(bits & SIGN_MASK | scaled)
}

/// `cbrt(z)·2^123` for `z = mantissa·2^(remainder−112) ∈ [1, 8)`, within
/// [`CBRT_GATE`]/2 units of 2^-123.
///
/// Pure unsigned fixed point end to end — an `f128` multiply is soft-float,
/// and even the f64 unit is dead weight here: a degree-2 Taylor seed and one
/// folded Newton step land `r = R·2^-63 ≈ z^(-1/3)` in a handful of 64-bit
/// products.  The seed is biased a hair below the true value so the residual
/// `h = r³z − 1` stays strictly negative and every limb stays unsigned; then
/// `sx = r²z` misses the cube root by `(1 + h)^(-2/3) ≈ 1 − ⅔h + 5⁄9h²`, and
/// both correction terms are short exact-width products.  With `|h| ≤ 2^-44`,
/// the dropped h³ term is below 2^-10 units and each shift truncation costs at
/// most a few units.
fn cbrt_fixed(mantissa: u128, remainder: i32) -> u128 {
    // Degree-2 Taylor expansion of m^(-1/3) from the nearest of 64 interval
    // centers c = 1 + (2i+1)/128: within 2^-23.2 of the true value, so the
    // Newton step below lands within 2^-46.4.
    #[allow(clippy::cast_possible_truncation)]
    let m64 = (mantissa >> 49) as u64; // m·2^63, truncated
    let i = (m64 >> 57) as usize & 63;
    #[allow(clippy::cast_possible_wrap)]
    let d = (m64 & ((1 << 57) - 1)) as i64 - (1 << 56); // (m − c)·2^63
    let slope = ((i128::from(d) * i128::from(SEED_SLOPE[i])) >> 65) as i64;
    #[allow(clippy::cast_sign_loss)]
    let dd = ((i128::from(d) * i128::from(d)) >> 63) as u64; // d²·2^63
    let curve = ((u128::from(dd) * u128::from(SEED_CURVE[i])) >> 66) as i64;
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    let r0 = (SEED_VALUE[i] as i64 - slope + curve) as u64; // m^(-1/3)·2^63

    // One Newton step r ← r·(4 − m·r³)·2^(-w/3)/3 folds the octave scaling
    // into its constant.  Its truncations push r at most a few units of 2^-63
    // up, so subtracting 64 units keeps r strictly below z^(-1/3).
    let r2 = mul_hi_64(r0, r0); // r0²·2^62
    let r3 = mul_hi_64(r2, r0); // r0³·2^61
    let t = (1 << 62) - mul_hi_64(m64, r3); // (4 − m·r0³)·2^60
    let u = mul_hi_64(r0, t); // r0·(4 − m·r0³)·2^59
    #[allow(clippy::cast_possible_truncation)]
    let r = (((u128::from(u) * u128::from(NEWTON[remainder as usize])) >> 61) as u64) - 64;

    let z = mantissa << (13 + remainder); // z·2^125, exact
    // r²z·2^123.  The units mhi_approx runs short cancel to a third of a unit:
    // h is measured from this same truncated sx, so the correction chases the
    // cube root of what sx actually is.
    let sx = mhi_approx(u128::from(r) * u128::from(r), z);
    // |h|·2^122 as one 64×128-bit product: r·sx = (1 + h)·2^122 with h < 0.
    let hp = (1 << 122)
        - (u128::from(r) * (sx >> 64) + ((u128::from(r) * (sx & u128::from(u64::MAX))) >> 64));

    // ⅔·2^128 rounded to nearest: 2^129 ≡ 2 (mod 3) makes the form exact.
    const TWO_THIRDS: u128 = 2 * (u128::MAX / 3) + 1;
    // 5⁄9·2^64; a floor is plenty for a term below 2^-93.
    const FIVE_NINTHS: u128 = (5 << 64) / 9;

    // ⅔sx·2^123, issued alongside hp so the linear term waits on only one
    // 128-bit product after the residual.
    let s23 = mhi_approx(sx, TWO_THIRDS);
    let (high, low) = wmul(hp, s23);
    let linear = (high << 6) | (low >> 122); // ⅔|h|·sx·2^123
    #[allow(clippy::cast_possible_truncation)]
    let hs = u128::from((hp >> 26) as u64); // |h|·2^96
    let quadratic = (((((hs * hs) >> 69) * FIVE_NINTHS) >> 64) * (sx >> 60)) >> 63;

    sx + linear + quadratic
}

/// Return the integer significand and unbiased exponent represented by a
/// normalized magnitude, including the virtual exponent used for subnormals.
#[inline]
const fn parts(magnitude: i128) -> (u128, i32) {
    (
        magnitude as u128 & MANTISSA_MASK | IMPLICIT_BIT,
        (magnitude >> EXP_SHIFT) as i32 - BIAS,
    )
}

/// Adjacent-float midpoints around `m * 2^(e-112)`, both expressed in units
/// of `2^(e-114)`.  Below an exact power of two the spacing is half as large.
#[inline]
const fn midpoints(m: u128) -> (u128, u128) {
    let center = m << 2;
    (center - if m == IMPLICIT_BIT { 1 } else { 2 }, center + 2)
}

/// Exact `a*b*c` as little-endian 128-bit limbs.
#[inline]
fn mul3(a: u128, b: u128, c: u128) -> [u128; 3] {
    let (ab_high, ab_low) = wmul(a, b);
    let (carry, low) = wmul(ab_low, c);
    let (mut high, middle) = wmul(ab_high, c);
    let (middle, overflow) = middle.overflowing_add(carry);
    high += u128::from(overflow);
    [low, middle, high]
}

/// Round an approximate reciprocal square root by exact midpoint tests.
fn correct_rsqrt(mantissa: u128, exponent: i32, mut candidate: f128) -> f128 {
    loop {
        let bits = candidate.to_bits();
        let (m, e) = parts(bits as i128);
        let (lower, upper) = midpoints(m);
        let one = shl_384([1, 0, 0], (340 - exponent - 2 * e) as u32);
        let odd = bits & 1 != 0;

        // 1/sqrt(x) is below L iff x*L^2 > 1.
        let side = cmp_384(mul3(mantissa, lower, lower), one);
        if side == Ordering::Greater || side == Ordering::Equal && odd {
            candidate = f128::from_bits(bits - 1);
            continue;
        }

        // 1/sqrt(x) is above U iff x*U^2 < 1.
        let side = cmp_384(mul3(mantissa, upper, upper), one);
        if side == Ordering::Less || side == Ordering::Equal && odd {
            candidate = f128::from_bits(bits + 1);
            continue;
        }
        return candidate;
    }
}

/// Round an approximate cube root of `mantissa * 2^(remainder-112)` by exact
/// midpoint tests.
fn correct_cbrt(mantissa: u128, remainder: i32, mut candidate: f128) -> f128 {
    loop {
        let bits = candidate.to_bits();
        let (m, e) = parts(bits as i128);
        let (lower, upper) = midpoints(m);
        let input = shl_384([mantissa, 0, 0], (remainder - 3 * e + 230) as u32);
        let odd = bits & 1 != 0;

        let side = cmp_384(mul3(lower, lower, lower), input);
        if side == Ordering::Greater || side == Ordering::Equal && odd {
            candidate = f128::from_bits(bits - 1);
            continue;
        }

        let side = cmp_384(mul3(upper, upper, upper), input);
        if side == Ordering::Less || side == Ordering::Equal && odd {
            candidate = f128::from_bits(bits + 1);
            continue;
        }
        return candidate;
    }
}

/// MPFR certification that [`CBRT_GATE`] covers [`cbrt_fixed`]'s true error
/// with the 2× margin the project requires.  Run with
/// `CC=clang cargo +nightly test --release --features "f128 mpfr"`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::*;
    use rug::{Float, ops::Pow};

    const PRECISION: u32 = 300;
    const SAMPLES: u64 = 200_000;

    fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Worst `|candidate − cbrt(z)·2^123| / CBRT_GATE` over random reduced
    /// arguments — exactly the units the gate compares.
    #[test]
    fn fast_candidate_is_sound() {
        let mut worst = 0.0;
        let mut worst_at = (0, 0);

        for i in 0..SAMPLES {
            let mantissa = (u128::from(mix(i)) << 64 | u128::from(mix(i ^ 0x9E37_79B9)))
                & MANTISSA_MASK
                | IMPLICIT_BIT;
            let remainder = (mix(i ^ 0xABCD) % 3) as i32;
            let candidate = cbrt_fixed(mantissa, remainder);
            let z = Float::with_val(PRECISION, mantissa)
                * Float::with_val(PRECISION, 2).pow(remainder - 112);
            let truth = z.cbrt() * Float::with_val(PRECISION, 2).pow(123);
            let slip: Float = truth - Float::with_val(PRECISION, candidate);
            let ratio = slip.abs().to_f64() / CBRT_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_at = (mantissa, remainder);
            }
        }
        println!(
            "cbrtq fast candidate: worst |err|/gate = {worst:.4} at m={:#x} w={}",
            worst_at.0, worst_at.1
        );
        assert!(
            worst < 0.5,
            "cbrtq gate covers only {:.2}× the slip at m={:#x} w={}",
            1.0 / worst,
            worst_at.0,
            worst_at.1
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_and_wide_product() {
        assert_eq!(
            mul3(u128::MAX, u128::MAX, u128::MAX),
            [u128::MAX, 2, u128::MAX - 2]
        );
        assert_eq!(sqrtq(4.0).to_bits(), 2.0_f128.to_bits());
        assert_eq!(rsqrtq(-0.0).to_bits(), f128::NEG_INFINITY.to_bits());
        assert_eq!(cbrtq(-8.0).to_bits(), (-2.0_f128).to_bits());
    }
}
