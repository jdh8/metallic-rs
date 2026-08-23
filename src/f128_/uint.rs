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

/// Compare two 384-bit little-endian values.
#[must_use]
#[inline]
pub fn cmp_384(a: [u128; 3], b: [u128; 3]) -> Ordering {
    a[2].cmp(&b[2]).then(a[1].cmp(&b[1])).then(a[0].cmp(&b[0]))
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
