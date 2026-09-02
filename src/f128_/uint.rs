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

/// Approximate high half of an unsigned 128×128-bit product from three 64-bit
/// products: up to two units of 2^-128 short of [`mhi`], never over.
#[must_use]
#[inline]
pub const fn mhi_approx(x: u128, y: u128) -> u128 {
    let (xh, xl) = (x >> 64, x & u64::MAX as u128);
    let (yh, yl) = (y >> 64, y & u64::MAX as u128);

    xh * yh + ((xh * yl) >> 64) + ((xl * yh) >> 64)
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

/// `a - b`, discarding the borrow out of the 384-bit window.
#[must_use]
#[inline]
pub fn sub_384(a: [u128; 3], b: [u128; 3]) -> [u128; 3] {
    let (low, borrow) = a[0].overflowing_sub(b[0]);
    let (middle, middle_borrow) = a[1].overflowing_sub(b[1]);
    let (middle, propagated) = middle.overflowing_sub(u128::from(borrow));
    let high = a[2]
        .wrapping_sub(b[2])
        .wrapping_sub(u128::from(middle_borrow))
        .wrapping_sub(u128::from(propagated));

    [low, middle, high]
}

/// Exact product of a 128-bit and a 256-bit value, little-endian.
#[must_use]
#[inline]
pub fn wmul_128x256(a: u128, b: [u128; 2]) -> [u128; 3] {
    let (high, low) = wmul(a, b[0]);
    let (top, middle) = wmul(a, b[1]);
    let (middle, carry) = middle.overflowing_add(high);

    [low, middle, top + u128::from(carry)]
}

/// Exact product of a 128-bit and a 384-bit value, little-endian.
#[must_use]
#[inline]
pub fn wmul_128x384(a: u128, b: [u128; 3]) -> [u128; 4] {
    let (high, low) = wmul(a, b[0]);
    let (middle_high, middle) = wmul(a, b[1]);
    let (top, third) = wmul(a, b[2]);
    let (middle, carry) = middle.overflowing_add(high);
    let (third, high_carry) = third.overflowing_add(middle_high + u128::from(carry));

    [low, middle, third, top + u128::from(high_carry)]
}

/// High 384 bits of an unsigned 384×384-bit product, little-endian.
///
/// The dropped tail makes the result up to four units of 2^-384 short of the
/// exact quotient `⌊a·b / 2^384⌋`.
#[must_use]
#[inline]
pub fn mul_hi_384(a: [u128; 3], b: [u128; 3]) -> [u128; 3] {
    let (h02, l02) = wmul(a[0], b[2]);
    let (h11, l11) = wmul(a[1], b[1]);
    let (h20, l20) = wmul(a[2], b[0]);
    let (h12, l12) = wmul(a[1], b[2]);
    let (h21, l21) = wmul(a[2], b[1]);
    let (h22, l22) = wmul(a[2], b[2]);

    // The diagonal one limb below the window contributes its high halves plus
    // the carry of its low halves; everything further down is the slack above.
    let (sum, carry_low) = l02.overflowing_add(l11);
    let (_, carry_high) = sum.overflowing_add(l20);
    let (diagonal, first) = h02.overflowing_add(h11);
    let (diagonal, second) = diagonal.overflowing_add(h20);
    let (diagonal, third) =
        diagonal.overflowing_add(u128::from(carry_low) + u128::from(carry_high));
    let spill = u128::from(first) + u128::from(second) + u128::from(third);

    add_384(
        add_384([l12, h12, 0], [l21, h21, 0]),
        add_384([0, l22, h22], [diagonal, spill, 0]),
    )
}

/// `x >> shift` on a 256-bit value, saturating to zero past the window.
#[must_use]
#[inline]
pub const fn shr_256_sat(x: [u128; 2], shift: u32) -> [u128; 2] {
    if shift >= 256 {
        return [0, 0];
    }
    if shift >= 128 {
        return [x[1] >> (shift - 128), 0];
    }
    // `<< 1 << (127 - shift)` is `<< (128 - shift)` with the no-op case in range.
    [
        (x[0] >> shift) | (x[1] << 1 << (127 - shift)),
        x[1] >> shift,
    ]
}

/// `x >> shift` on a 384-bit value, saturating to zero past the window.
#[must_use]
#[inline]
pub fn shr_384_sat(x: [u128; 3], shift: u32) -> [u128; 3] {
    shr_sat(x, shift)
}

/// `x >> shift` on an `N`-limb little-endian value, saturating to zero past
/// the window.
#[must_use]
#[inline]
pub fn shr_sat<const N: usize>(x: [u128; N], shift: u32) -> [u128; N] {
    let mut result = [0; N];

    if shift >= 128 * N as u32 {
        return result;
    }
    let word = (shift / 128) as usize;
    let bits = shift % 128;

    for i in word..N {
        let carry = match x.get(i + 1) {
            Some(&next) if bits != 0 => next << (128 - bits),
            _ => 0,
        };
        result[i - word] = (x[i] >> bits) | carry;
    }
    result
}

/// Whether any of the low `n` bits of an `N`-limb little-endian value is set,
/// for `n ≤ 128·N`.
#[must_use]
#[inline]
pub fn any_below<const N: usize>(x: [u128; N], n: u32) -> bool {
    let word = (n / 128) as usize;
    let bits = n % 128;

    x[..word].iter().any(|&limb| limb != 0)
        || (bits != 0 && x[word] & (u128::MAX >> (128 - bits)) != 0)
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

/// The top 64 bits of `(high:low) << shift`, for `shift < 64`.  LLVM has no
/// 128-bit funnel shift, so a `u128` shift pair would cost a `shld`, a plain
/// shift and a `cmov`; cut from 64-bit limbs it is one `shld`.
#[must_use]
#[inline]
pub const fn funnel(low: u64, high: u64, shift: u32) -> u64 {
    (high << shift) | (low >> 1 >> (63 - shift))
}

/// The low 64 bits of `(high:low) >> shift`, for `shift < 64` — [`funnel`]'s
/// mirror, one `shrd`.
#[must_use]
#[inline]
pub const fn funnel_down(low: u64, high: u64, shift: u32) -> u64 {
    (low >> shift) | (high << 1 << (63 - shift))
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

        let mut state = 0x9E37_79B9_7F4A_7C15_F39C_C060_5CED_C834_u128;
        for _ in 0..1000 {
            state = state.wrapping_mul(0x2545_F491_4F6C_DD1D).rotate_left(43);
            let x = state;
            state = state.wrapping_mul(0x2545_F491_4F6C_DD1D).rotate_left(43);
            let y = state;
            let deficit = mhi(x, y) - mhi_approx(x, y);
            assert!(deficit <= 2, "{x:#x} {y:#x}: {deficit}");
        }
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
    fn wide_multiply() {
        let a = [0x1234, u128::MAX, 0x9abc];
        let b = [u128::MAX, 0x5678, u128::MAX];

        // Reference: assemble both operands and the product from 64-bit limbs.
        let exact = |x: [u128; 3], y: [u128; 3]| {
            let digit =
                |v: [u128; 3], i: u32| v[(i / 2) as usize] >> (64 * (i % 2)) & u128::from(u64::MAX);
            let mut wide = [0u128; 12];
            for i in 0..6 {
                let mut carry = 0;
                for j in 0..6 {
                    let t = digit(x, i) * digit(y, j) + wide[(i + j) as usize] + carry;
                    wide[(i + j) as usize] = t & u128::from(u64::MAX);
                    carry = t >> 64;
                }
                wide[(i + 6) as usize] = carry;
            }
            wide
        };
        let wide = exact(a, b);
        let high = [
            wide[6] | wide[7] << 64,
            wide[8] | wide[9] << 64,
            wide[10] | wide[11] << 64,
        ];
        let computed = mul_hi_384(a, b);
        let deficit = sub_384(high, computed);
        assert!(
            deficit[1] == 0 && deficit[2] == 0 && deficit[0] < 4,
            "{deficit:?}"
        );

        assert_eq!(
            wmul_128x256(u128::MAX, [u128::MAX, u128::MAX]),
            [1, u128::MAX, u128::MAX - 1]
        );
        assert_eq!(
            wmul_128x384(u128::MAX, [u128::MAX, u128::MAX, u128::MAX]),
            [1, u128::MAX, u128::MAX, u128::MAX - 1]
        );
        assert_eq!(
            wmul_128x384(3, [u128::MAX / 5, 0, 1 << 100]),
            [u128::MAX / 5 * 3, 0, 3 << 100, 0]
        );
    }

    #[test]
    fn saturating_shifts() {
        let x = [0x1234, 0x5678, 0x9abc];
        assert_eq!(shr_384_sat(x, 0), x);
        assert_eq!(shr_384_sat(x, 128), [0x5678, 0x9abc, 0]);
        assert_eq!(
            shr_384_sat(x, 4),
            [
                0x8000_0000_0000_0000_0000_0000_0000_0123,
                0xc000_0000_0000_0000_0000_0000_0000_0567,
                0x9ab
            ]
        );
        assert_eq!(shr_384_sat(x, 384), [0; 3]);
        assert_eq!(shr_384_sat(x, 500), [0; 3]);
        assert_eq!(shr_256_sat([0x1234, 0x5678], 0), [0x1234, 0x5678]);
        assert_eq!(
            shr_256_sat([0x1234, 0x5678], 4),
            [0x8000_0000_0000_0000_0000_0000_0000_0123, 0x567]
        );
        assert_eq!(shr_256_sat([0x1234, 0x5678], 130), [0x159e, 0]);
        assert_eq!(shr_256_sat([0x1234, 0x5678], 256), [0, 0]);
        assert!(any_below([0, 1, 0], 129));
        assert!(!any_below([0, 1, 0], 128));
        assert!(any_below([1, 0, 0], 1));
        assert!(!any_below([2, 0, 0], 1));
    }

    #[test]
    fn wide_subtract() {
        assert_eq!(sub_384([0, 0, 1], [1, 0, 0]), [u128::MAX, u128::MAX, 0]);
        assert_eq!(sub_384([5, 6, 7], [1, 2, 3]), [4, 4, 4]);
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
