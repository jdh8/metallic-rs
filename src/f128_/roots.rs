use super::uint::{cmp_384, mhi_approx, mul_hi_64, shl_384, wmul};
use super::{
    BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, MANTISSA_MASK, QUIET_BIT, SIGN_MASK, cbrt_tables,
    rsqrt_tables, split,
};
use core::cmp::Ordering;

/// Half-width, in units of the candidate's 2^-125, of the rounding-tie window
/// [`sqrtq`] refuses to decide from the fast candidate and hands to the exact
/// midpoint walk instead.  [`sqrt_fixed`] is analytically within a handful of
/// units of the true square root; [`ziv_soundness`] certifies the ≥ 2× margin
/// the project requires.
const SQRT_GATE: u128 = 32;

/// The square root.
#[must_use]
#[inline]
pub fn sqrtq(x: f128) -> f128 {
    let bits = x.to_bits();
    let magnitude = bits & !SIGN_MASK;

    if magnitude > EXP_MASK {
        return f128::from_bits(bits | QUIET_BIT);
    }
    if magnitude == 0 {
        return x;
    }
    if bits & SIGN_MASK != 0 {
        return f128::NAN;
    }
    if magnitude == EXP_MASK {
        return x;
    }

    let (mantissa, exponent) = split(magnitude);
    let w = exponent.rem_euclid(2);
    let scale = exponent.div_euclid(2);

    let candidate = sqrt_fixed(mantissa, w);
    // Round the Q125 candidate to 113 bits.  Composing the bits by adding the
    // significand onto a BIAS − 1 exponent field lets a carry out of the
    // rounding land on an exact 2 instead of overflowing the mantissa; a
    // square root is always normal, so the 2^scale factor folds into the same
    // field for free.
    let rounded =
        f128::from_bits((((BIAS - 1 + scale) as u128) << EXP_SHIFT) + ((candidate + 4096) >> 13));
    let rest = candidate & 8191;

    if rest.abs_diff(4096) <= SQRT_GATE {
        correct_sqrt(mantissa, exponent, rounded)
    } else {
        rounded
    }
}

/// `sqrt(z)·2^125` for `z = mantissa·2^(w−112) ∈ [1, 4)`, within
/// [`SQRT_GATE`]/2 units of 2^-125.
// The `#[inline]` entry points are instantiated in the caller's crate, so their
// private helpers must be inline candidates too or every call crosses the GOT.
#[inline]
fn sqrt_fixed(mantissa: u128, w: i32) -> u128 {
    sqrt_wide(mantissa << (14 + w))
}

/// `sqrt(z)·2^125` for a full-width fixed-point `z·2^126 ∈ [2^126, 2^128)`,
/// within [`SQRT_GATE`]/2 units of 2^-125 — the frame behind [`sqrt_fixed`],
/// also fed a 127-bit sum of squares by [`super::hypot::hypotq`].
///
/// The same frame as [`rsqrt_fixed`] on the same seed: `r = R·2^-63 ≈
/// z^(-1/2)` from [`rsqrt64`], then `s = r·z` misses the square root by
/// `(1 + h)^(-1/2) ≈ 1 + |h|/2 + ⅜h²` with `h = r²z − 1` strictly negative,
/// and both correction terms are short exact-width products.  With
/// `|h| ≤ 2^-42`, the dropped 5⁄16·|h|³ term is below 2^-1 units and each
/// shift truncation costs at most a few units.  The seed reads only the top
/// 64 bits, so bits past the 113th cost it nothing.
// `always`: two callers (`sqrt_fixed`, `hypot_fixed`), and without it `hypotq`'s
// fast path reached this wrapper through a GOT-indirect call.
#[inline(always)]
pub(super) fn sqrt_wide(z: u128) -> u128 {
    sqrt_wide_seeded(z).0
}

/// [`sqrt_wide`] together with its seed `r = R·2^-63 ≈ z^(-1/2)` (strictly
/// below the true value, within ~2^-43): `1/(2√z)` for one more Newton step
/// on a wider residual, which is how [`super::asin`] reaches 2^-160 without a
/// second seed or a divide.
#[inline]
pub(super) fn sqrt_wide_seeded(z: u128) -> (u128, u64) {
    #[allow(clippy::cast_possible_truncation)]
    let w = (z >> 127) as i32;
    let r = rsqrt64(z >> (14 + w), w);
    // |h|·2^124: the square of r is exact in u128 and the 128×128 high product
    // runs at most two units short, an overshoot of |h| the gate absorbs.
    let hp = (1 << 124) - mhi_approx(u128::from(r) * u128::from(r), z);

    // s·2^125 = r·z as one 64×128-bit product, truncated.
    let s0 = u128::from(r) * (z >> 64) + ((u128::from(r) * (z & u128::from(u64::MAX))) >> 64);

    // s·|h|/2·2^125 = s0·hp·2^-125.  Unlike [`rsqrt_fixed`], whose series
    // multiplier is exactly the stored `r`, the multiplier here must carry the
    // full width of `s0`: a 64-bit truncation of `s` costs `2^-63·|h|·2^249 ≈
    // 2^17` units.  `hp < 2^81`, so pre-shifting by 3 cannot overflow.
    let linear = mhi_approx(s0, hp << 3);
    #[allow(clippy::cast_possible_truncation)]
    let hs = u128::from((hp >> 28) as u64); // |h|·2^96
    #[allow(clippy::cast_possible_truncation)]
    let s64 = (s0 >> 62) as u64; // s·2^63, plenty for the quadratic term
    // ⅜h²s·2^125; 3/8 is dyadic, so the shifts fold it in exactly.
    let quadratic = (((hs * hs) >> 68) * 3 * u128::from(s64)) >> 65;

    (s0 + linear + quadratic, r)
}

/// Half-width, in units of the candidate's 2^-124, of the rounding-tie window
/// [`rsqrtq`] refuses to decide from the fast candidate and hands to the exact
/// midpoint walk instead.  [`rsqrt_fixed`] is analytically within a handful of
/// units of the true reciprocal square root; [`ziv_soundness`] certifies the
/// ≥ 2× margin the project requires.
const RSQRT_GATE: u128 = 32;

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

    let (mantissa, exponent) = split(magnitude);
    let w = exponent.rem_euclid(2);
    let scale = -exponent.div_euclid(2);

    let candidate = rsqrt_fixed(mantissa, w);
    // Round the Q124 candidate to 113 bits.  Composing the bits by adding the
    // significand onto an exponent field two below the result's lets a carry
    // out of the rounding land on an exact power of two instead of overflowing
    // the mantissa; a reciprocal square root is always normal, so the 2^scale
    // factor folds into the same field for free.
    let rounded =
        f128::from_bits((((BIAS - 2 + scale) as u128) << EXP_SHIFT) + ((candidate + 1024) >> 11));
    let rest = candidate & 2047;

    if rest.abs_diff(1024) <= RSQRT_GATE {
        correct_rsqrt(mantissa, exponent, rounded)
    } else {
        rounded
    }
}

/// `rsqrt(z)·2^124` for `z = mantissa·2^(w−112) ∈ [1, 4)`, within
/// [`RSQRT_GATE`]/2 units of 2^-124.
///
/// Pure unsigned fixed point end to end, on the same frame as [`cbrt_fixed`]:
/// a degree-2 Taylor seed and one folded Newton step land `r = R·2^-63 ≈
/// z^(-1/2)` in a handful of 64-bit products.  The seed is biased a hair below
/// the true value so the residual `h = r²z − 1` stays strictly negative and
/// every limb stays unsigned; then `r` misses the reciprocal square root by
/// `(1 + h)^(-1/2) ≈ 1 + |h|/2 + ⅜h²`, and both correction terms are short
/// exact-width products.  With `|h| ≤ 2^-42`, the dropped 5⁄16·h³ term is
/// below 2^-4 units and each shift truncation costs at most a few units.
#[inline]
fn rsqrt_fixed(mantissa: u128, w: i32) -> u128 {
    let r = rsqrt64(mantissa, w);
    let z = mantissa << (14 + w); // z·2^126, exact
    // |h|·2^124: the square of r is exact in u128 and the 128×128 high product
    // runs at most two units short, an overshoot of |h| the gate absorbs.
    let hp = (1 << 124) - mhi_approx(u128::from(r) * u128::from(r), z);

    // r·|h|/2·2^124 as one 64×128-bit product.
    let linear = u128::from(r) * (hp >> 64) + ((u128::from(r) * (hp & u128::from(u64::MAX))) >> 64);
    #[allow(clippy::cast_possible_truncation)]
    let hs = u128::from((hp >> 28) as u64); // |h|·2^96
    // ⅜h²r·2^124; 3/8 is dyadic, so the shifts fold it in exactly.
    let quadratic = (((hs * hs) >> 68) * 3 * u128::from(r)) >> 66;

    (u128::from(r) << 61) + linear + quadratic
}

/// `z^(-1/2)·2^63` for `z = mantissa·2^(w−112) ∈ [1, 4)`, strictly below the
/// true value and within ~2^-43 of it — the shared seed of [`rsqrt_fixed`]
/// and [`sqrt_fixed`].
#[inline]
fn rsqrt64(mantissa: u128, w: i32) -> u64 {
    // Degree-2 Taylor expansion of m^(-1/2) from the nearest of 64 interval
    // centers c = 1 + (2i+1)/128: within 2^-22.6 of the true value, so the
    // Newton step below lands within 2^-43.7.
    #[allow(clippy::cast_possible_truncation)]
    let m64 = (mantissa >> 49) as u64; // m·2^63, truncated
    let i = (m64 >> 57) as usize & 63;
    #[allow(clippy::cast_possible_wrap)]
    let d = (m64 & ((1 << 57) - 1)) as i64 - (1 << 56); // (m − c)·2^63
    let slope = ((i128::from(d) * i128::from(rsqrt_tables::SEED_SLOPE[i])) >> 65) as i64;
    #[allow(clippy::cast_sign_loss)]
    let dd = ((i128::from(d) * i128::from(d)) >> 63) as u64; // d²·2^63
    let curve = ((u128::from(dd) * u128::from(rsqrt_tables::SEED_CURVE[i])) >> 65) as i64;
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    let r0 = (rsqrt_tables::SEED_VALUE[i] as i64 - slope + curve) as u64; // m^(-1/2)·2^63

    // One Newton step r ← r·(3 − m·r²)·2^(-w/2)/2 folds the halving and the
    // octave scaling into its constant.  Its truncations push r at most a few
    // units of 2^-63 up, so subtracting 64 units keeps r strictly below
    // z^(-1/2).
    let r2 = mul_hi_64(r0, r0); // r0²·2^62
    let t = (3 << 61) - mul_hi_64(m64, r2); // (3 − m·r0²)·2^61
    let u = mul_hi_64(r0, t); // r0·(3 − m·r0²)·2^60
    #[allow(clippy::cast_possible_truncation)]
    {
        (((u128::from(u) * u128::from(rsqrt_tables::NEWTON[w as usize])) >> 61) as u64) - 64
    }
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
#[inline]
fn cbrt_fixed(mantissa: u128, remainder: i32) -> u128 {
    // Degree-2 Taylor expansion of m^(-1/3) from the nearest of 64 interval
    // centers c = 1 + (2i+1)/128: within 2^-23.2 of the true value, so the
    // Newton step below lands within 2^-46.4.
    #[allow(clippy::cast_possible_truncation)]
    let m64 = (mantissa >> 49) as u64; // m·2^63, truncated
    let i = (m64 >> 57) as usize & 63;
    #[allow(clippy::cast_possible_wrap)]
    let d = (m64 & ((1 << 57) - 1)) as i64 - (1 << 56); // (m − c)·2^63
    let slope = ((i128::from(d) * i128::from(cbrt_tables::SEED_SLOPE[i])) >> 65) as i64;
    #[allow(clippy::cast_sign_loss)]
    let dd = ((i128::from(d) * i128::from(d)) >> 63) as u64; // d²·2^63
    let curve = ((u128::from(dd) * u128::from(cbrt_tables::SEED_CURVE[i])) >> 66) as i64;
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    let r0 = (cbrt_tables::SEED_VALUE[i] as i64 - slope + curve) as u64; // m^(-1/3)·2^63

    // One Newton step r ← r·(4 − m·r³)·2^(-w/3)/3 folds the octave scaling
    // into its constant.  Its truncations push r at most a few units of 2^-63
    // up, so subtracting 64 units keeps r strictly below z^(-1/3).
    let r2 = mul_hi_64(r0, r0); // r0²·2^62
    let r3 = mul_hi_64(r2, r0); // r0³·2^61
    let t = (1 << 62) - mul_hi_64(m64, r3); // (4 − m·r0³)·2^60
    let u = mul_hi_64(r0, t); // r0·(4 − m·r0³)·2^59
    #[allow(clippy::cast_possible_truncation)]
    let r =
        (((u128::from(u) * u128::from(cbrt_tables::NEWTON[remainder as usize])) >> 61) as u64) - 64;

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

/// Round an approximate square root of `mantissa * 2^(exponent-112)` by exact
/// midpoint tests: `sqrt(x)` is below `L` iff `x < L²`, and `L²` is a single
/// exact [`wmul`].
fn correct_sqrt(mantissa: u128, exponent: i32, mut candidate: f128) -> f128 {
    loop {
        let bits = candidate.to_bits();
        let (m, e) = parts(bits as i128);
        let (lower, upper) = midpoints(m);
        // x vs L² = l²·2^(2e−228): scale both by 2^(228−2e), so the input
        // side is mantissa·2^(116 + exponent − 2e) with the shift in [114, 117].
        let input = shl_384([mantissa, 0, 0], (116 + exponent - 2 * e) as u32);
        let odd = bits & 1 != 0;

        let (high, low) = wmul(lower, lower);
        let side = cmp_384(input, [low, high, 0]);
        if side == Ordering::Less || side == Ordering::Equal && odd {
            candidate = f128::from_bits(bits - 1);
            continue;
        }

        let (high, low) = wmul(upper, upper);
        let side = cmp_384(input, [low, high, 0]);
        if side == Ordering::Greater || side == Ordering::Equal && odd {
            candidate = f128::from_bits(bits + 1);
            continue;
        }
        return candidate;
    }
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

/// MPFR certification that [`SQRT_GATE`], [`CBRT_GATE`] and [`RSQRT_GATE`]
/// cover the true errors of [`sqrt_fixed`], [`cbrt_fixed`] and
/// [`rsqrt_fixed`] with the 2× margin the project requires.  Run with
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

    /// Worst `|candidate − sqrt(z)·2^125| / SQRT_GATE` over random reduced
    /// arguments — exactly the units the gate compares.
    #[test]
    fn sqrt_candidate_is_sound() {
        let mut worst = 0.0;
        let mut worst_at = (0, 0);

        for i in 0..SAMPLES {
            let mantissa = (u128::from(mix(i)) << 64 | u128::from(mix(i ^ 0x9E37_79B9)))
                & MANTISSA_MASK
                | IMPLICIT_BIT;
            let w = (mix(i ^ 0xABCD) % 2) as i32;
            let candidate = sqrt_fixed(mantissa, w);
            let z =
                Float::with_val(PRECISION, mantissa) * Float::with_val(PRECISION, 2).pow(w - 112);
            let truth = z.sqrt() * Float::with_val(PRECISION, 2).pow(125);
            let slip: Float = truth - Float::with_val(PRECISION, candidate);
            let ratio = slip.abs().to_f64() / SQRT_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_at = (mantissa, w);
            }
        }
        println!(
            "sqrtq fast candidate: worst |err|/gate = {worst:.4} at m={:#x} w={}",
            worst_at.0, worst_at.1
        );
        assert!(
            worst < 0.5,
            "sqrtq gate covers only {:.2}× the slip at m={:#x} w={}",
            1.0 / worst,
            worst_at.0,
            worst_at.1
        );
    }

    /// Worst `|candidate − rsqrt(z)·2^124| / RSQRT_GATE` over random reduced
    /// arguments — exactly the units the gate compares.
    #[test]
    fn rsqrt_candidate_is_sound() {
        let mut worst = 0.0;
        let mut worst_at = (0, 0);

        for i in 0..SAMPLES {
            let mantissa = (u128::from(mix(i)) << 64 | u128::from(mix(i ^ 0x9E37_79B9)))
                & MANTISSA_MASK
                | IMPLICIT_BIT;
            let w = (mix(i ^ 0xABCD) % 2) as i32;
            let candidate = rsqrt_fixed(mantissa, w);
            let z =
                Float::with_val(PRECISION, mantissa) * Float::with_val(PRECISION, 2).pow(w - 112);
            let truth = z.recip_sqrt() * Float::with_val(PRECISION, 2).pow(124);
            let slip: Float = truth - Float::with_val(PRECISION, candidate);
            let ratio = slip.abs().to_f64() / RSQRT_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_at = (mantissa, w);
            }
        }
        println!(
            "rsqrtq fast candidate: worst |err|/gate = {worst:.4} at m={:#x} w={}",
            worst_at.0, worst_at.1
        );
        assert!(
            worst < 0.5,
            "rsqrtq gate covers only {:.2}× the slip at m={:#x} w={}",
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
