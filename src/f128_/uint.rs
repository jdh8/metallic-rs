// This module is compiled only by the nightly-only `f128` feature.
#![allow(clippy::incompatible_msrv)]

use core::cmp::Ordering;

/// Full unsigned 128×128-bit product as `(high, low)`.
#[must_use]
#[inline]
pub fn wmul(x: u128, y: u128) -> (u128, u128) {
    let (low, high) = x.carrying_mul(y, 0);
    (high, low)
}

/// High half of an unsigned 128×128-bit product.
#[must_use]
#[inline]
pub fn mhi(x: u128, y: u128) -> u128 {
    x.carrying_mul(y, 0).1
}

/// High half of an unsigned 64×64-bit product.
#[must_use]
#[inline]
pub const fn mul_hi_64(x: u64, y: u64) -> u64 {
    ((x as u128 * y as u128) >> 64) as u64
}

/// Exact `x²` as little-endian 128-bit limbs.
#[must_use]
#[inline]
pub fn sqr_384(x: u128) -> [u128; 3] {
    let (high, low) = wmul(x, x);
    [low, high, 0]
}

/// `x << shift`, discarding whatever leaves the 384-bit window.
#[must_use]
#[inline]
pub fn shl_384(x: [u128; 3], shift: u32) -> [u128; 3] {
    debug_assert!(shift < 384);
    let word = (shift / 128) as usize;
    let bits = shift % 128;
    let mut result = [0; 3];

    for i in (word..result.len()).rev() {
        let carry = if bits == 0 || i == word {
            0
        } else {
            x[i - word - 1] >> (128 - bits)
        };
        result[i] = (x[i - word] << bits) | carry;
    }
    result
}

/// `a + b`, discarding the carry out of the 384-bit window.
#[must_use]
#[inline]
pub fn add_384(a: [u128; 3], b: [u128; 3]) -> [u128; 3] {
    let (low, carry) = a[0].overflowing_add(b[0]);
    let (middle, middle_carry) = a[1].overflowing_add(b[1]);
    let (middle, propagated) = middle.overflowing_add(u128::from(carry));
    let high = a[2]
        .wrapping_add(b[2])
        .wrapping_add(u128::from(middle_carry))
        .wrapping_add(u128::from(propagated));

    [low, middle, high]
}

/// `-x` on a 384-bit little-endian two's complement value.
#[must_use]
#[inline]
pub fn neg_384(x: [u128; 3]) -> [u128; 3] {
    let (low, borrow) = 0_u128.overflowing_sub(x[0]);
    let (middle, middle_borrow) = 0_u128.overflowing_sub(x[1]);
    let (middle, propagated) = middle.overflowing_sub(u128::from(borrow));
    let high = 0_u128
        .wrapping_sub(x[2])
        .wrapping_sub(u128::from(middle_borrow))
        .wrapping_sub(u128::from(propagated));

    [low, middle, high]
}

/// `a + b`, discarding the carry out of the 256-bit window.
#[must_use]
#[inline]
pub fn add_256(a: [u128; 2], b: [u128; 2]) -> [u128; 2] {
    let (low, carry) = a[0].overflowing_add(b[0]);
    [low, a[1].wrapping_add(b[1]).wrapping_add(u128::from(carry))]
}

/// `a - b`, discarding the borrow out of the 256-bit window.
#[must_use]
#[inline]
pub fn sub_256(a: [u128; 2], b: [u128; 2]) -> [u128; 2] {
    let (low, borrow) = a[0].overflowing_sub(b[0]);
    [
        low,
        a[1].wrapping_sub(b[1]).wrapping_sub(u128::from(borrow)),
    ]
}

/// High 256 bits of an unsigned 256×256-bit product, little-endian.
///
/// The dropped tail makes the result up to two units of 2^-256 short of the
/// exact quotient `⌊a·b / 2^256⌋`.
#[must_use]
#[inline]
pub fn mul_hi_256(a: [u128; 2], b: [u128; 2]) -> [u128; 2] {
    let (high, low) = wmul(a[1], b[1]);
    let (cross_high, cross_low) = wmul(a[1], b[0]);
    let (other_high, other_low) = wmul(a[0], b[1]);

    // The two cross terms and the top of a[0]·b[0] all land one limb below the
    // window, so only their carry into it survives.
    let (_, carry) = cross_low.overflowing_add(other_low);
    let (_, tail_carry) = cross_low
        .wrapping_add(other_low)
        .overflowing_add(mhi(a[0], b[0]));
    let (middle, overflow) = cross_high.overflowing_add(other_high);
    let (middle, spill) = middle.overflowing_add(u128::from(carry) + u128::from(tail_carry));
    let (low, final_carry) = low.overflowing_add(middle);

    [
        low,
        high + u128::from(overflow) + u128::from(spill) + u128::from(final_carry),
    ]
}

/// `x << shift`, discarding whatever leaves the 256-bit window.
#[must_use]
#[inline]
pub fn shl_256(x: [u128; 2], shift: u32) -> [u128; 2] {
    debug_assert!(shift < 256);

    if shift >= 128 {
        return [0, x[0] << (shift - 128)];
    }
    // `>> 1 >> (127 - shift)` is `>> (128 - shift)` with the no-op case in range.
    [
        x[0] << shift,
        (x[1] << shift) | (x[0] >> 1 >> (127 - shift)),
    ]
}

/// Compare two 384-bit little-endian values.
#[must_use]
#[inline]
pub fn cmp_384(a: [u128; 3], b: [u128; 3]) -> Ordering {
    a[2].cmp(&b[2]).then(a[1].cmp(&b[1])).then(a[0].cmp(&b[0]))
}

/// Leading zeros of a 256-bit little-endian value; 256 when it is zero.
#[must_use]
#[inline]
pub const fn leading_zeros_256(x: [u128; 2]) -> u32 {
    if x[1] != 0 {
        x[1].leading_zeros()
    } else {
        128 + x[0].leading_zeros()
    }
}

/// Leading zeros of a 384-bit little-endian value; 384 when it is zero.
#[must_use]
#[inline]
pub const fn leading_zeros_384(x: [u128; 3]) -> u32 {
    if x[2] != 0 {
        x[2].leading_zeros()
    } else if x[1] != 0 {
        128 + x[1].leading_zeros()
    } else {
        256 + x[0].leading_zeros()
    }
}

/// Bits `[shift, shift + 128)` of a 384-bit little-endian value.
#[must_use]
#[inline]
pub fn extract_u128(x: [u128; 3], shift: u32) -> u128 {
    debug_assert!(shift < 384);
    let word = (shift / 128) as usize;
    let bits = shift % 128;
    let high = match x.get(word + 1) {
        Some(&next) if bits != 0 => next << (128 - bits),
        _ => 0,
    };

    (x[word] >> bits) | high
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_product() {
        assert_eq!(wmul(u128::MAX, u128::MAX), (u128::MAX - 1, 1));
        assert_eq!(mhi(u128::MAX, u128::MAX), u128::MAX - 1);
        assert_eq!(sqr_384(u128::MAX), [1, u128::MAX - 1, 0]);
    }

    #[test]
    fn wide_shift_and_add() {
        assert_eq!(shl_384([1, 0, 0], 0), [1, 0, 0]);
        assert_eq!(shl_384([1, 0, 0], 129), [0, 2, 0]);
        assert_eq!(
            shl_384([u128::MAX, 0, 0], 64),
            [u128::MAX << 64, u64::MAX.into(), 0]
        );
        assert_eq!(shl_384([1, 0, 0], 383), [0, 0, 1 << 127]);
        assert_eq!(add_384([u128::MAX, u128::MAX, 0], [1, 0, 0]), [0, 0, 1]);
        assert_eq!(shl_256([1, 0], 0), [1, 0]);
        assert_eq!(shl_256([1, 0], 129), [0, 2]);
        assert_eq!(shl_256([u128::MAX, 0], 1), [u128::MAX - 1, 1]);
        assert_eq!(leading_zeros_256([0, 0]), 256);
        assert_eq!(leading_zeros_256([u128::MAX, 0]), 128);
        assert_eq!(leading_zeros_384([0, 0, 0]), 384);
        assert_eq!(leading_zeros_384([u128::MAX, 0, 0]), 256);
        assert_eq!(leading_zeros_384([0, 0, 1]), 127);
        assert_eq!(cmp_384([0, 1, 0], [u128::MAX, 0, 0]), Ordering::Greater);
    }

    #[test]
    fn wide_extract() {
        let x = [0x1234, 0x5678, 0x9abc];
        assert_eq!(extract_u128(x, 0), 0x1234);
        assert_eq!(extract_u128(x, 128), 0x5678);
        assert_eq!(extract_u128(x, 256), 0x9abc);
        assert_eq!(extract_u128(x, 4), 0x5678 << 124 | 0x123);
        assert_eq!(extract_u128([0, 1, 0], 1), 1 << 127);
    }
}
