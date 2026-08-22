use super::{EXP_SHIFT, Magnitude, fma128, ldexp, normalize, wmul};
use core::cmp::Ordering;

const SIGN_MASK: u128 = 1 << 127;
const EXP_MASK: u128 = 0x7fff << EXP_SHIFT;
const MANTISSA_MASK: u128 = (1 << EXP_SHIFT) - 1;
const IMPLICIT_BIT: u128 = 1 << EXP_SHIFT;
const QUIET_BIT: u128 = 1 << (EXP_SHIFT - 1);
const BIAS: i32 = 0x3fff;

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

/// The cube root.
#[must_use]
#[inline]
pub fn cbrtq(x: f128) -> f128 {
    if x.is_nan() {
        return f128::from_bits(x.to_bits() | QUIET_BIT);
    }

    let (_, Magnitude::Normalized(magnitude)) = normalize(x) else {
        return x;
    };
    let (mantissa, exponent) = parts(magnitude);
    let remainder = exponent.rem_euclid(3);
    let scale = exponent.div_euclid(3);

    // Reduce |x| to z in [1, 8).  The existing f64 cbrt supplies about 53
    // seed bits without introducing another fitted table.
    let z = f128::from_bits(((BIAS + remainder) as u128) << EXP_SHIFT | (mantissa & MANTISSA_MASK));
    let seed = f64::from_bits(
        ((0x3ff + remainder) as u64) << 52 | ((mantissa & MANTISSA_MASK) >> 60) as u64,
    );
    let mut y = crate::cbrt(seed) as f128;

    // One ordinary Newton step reaches about 106 bits.  Evaluate the next
    // cube residual with FMAs so its correction carries roughly twice that.
    y = fma128(2.0, y, z / (y * y)) / 3.0;
    let y2 = y * y;
    let y2_low = fma128(y, y, -y2);
    let y3 = y2 * y;
    let y3_low = fma128(y, y2_low, fma128(y, y2, -y3));
    let candidate = y - ((y3 - z) + y3_low) / (3.0 * y2);
    let candidate = correct_cbrt(mantissa, remainder, candidate);

    ldexp(candidate, scale).copysign(x)
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

#[inline]
fn shl_384(x: u128, shift: u32) -> [u128; 3] {
    debug_assert!(shift < 384);
    let mut result = [0; 3];
    let word = (shift / 128) as usize;
    let bits = shift % 128;
    result[word] = x << bits;
    if bits != 0 && word + 1 < result.len() {
        result[word + 1] = x >> (128 - bits);
    }
    result
}

#[inline]
fn bit_384(bit: u32) -> [u128; 3] {
    debug_assert!(bit < 384);
    let mut result = [0; 3];
    result[(bit / 128) as usize] = 1 << (bit % 128);
    result
}

#[inline]
fn cmp_384(a: [u128; 3], b: [u128; 3]) -> Ordering {
    a[2].cmp(&b[2]).then(a[1].cmp(&b[1])).then(a[0].cmp(&b[0]))
}

/// Round an approximate reciprocal square root by exact midpoint tests.
fn correct_rsqrt(mantissa: u128, exponent: i32, mut candidate: f128) -> f128 {
    loop {
        let bits = candidate.to_bits();
        let (m, e) = parts(bits as i128);
        let (lower, upper) = midpoints(m);
        let one = bit_384((340 - exponent - 2 * e) as u32);
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
        let input = shl_384(mantissa, (remainder - 3 * e + 230) as u32);
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
