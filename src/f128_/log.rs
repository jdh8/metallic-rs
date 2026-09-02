//! The binary128 logarithms: natural, base 2, and base 10.
//!
//! The reduction is done in *log space*, so the table of logarithms to add back
//! is the only table the sum needs — and the only thing that knows the base.
//! One engine serves every base through [`Base`]: the estimate, the exact
//! reduction, and both frames are shared, while the per-exponent constant
//! (`log_b 2`), the logarithms of the rounded reciprocals, and the Taylor
//! coefficients `log_b(e)/(k+1)` come from the base's tables.
//!
//! 1. **Estimate.** A linear fit per 2<sup>-8</sup> bucket of the significand
//!    gives `j ≈ 2^18·log2(m)` to within nine tenths of a unit — enough to pick
//!    a reduction, not to be one.
//! 2. **Reduce.** The three 6-bit slices of `j` index tables of
//!    2<sup>-j/64</sup>, 2<sup>-j/4096</sup> and 2<sup>-j/262144</sup> rounded
//!    to 31 bits.  Their product is 93 bits wide, so `m` times it is a *single
//!    exact* 128×128-bit product, and `z = m·2^(-j/2^18) − 1` — under 2^-18.7 —
//!    comes out of it with every bit intact.
//! 3. **Add back.** `log(x) = e·ln2 + LOG0[j0] + LOG1[j1] + LOG2[j2] +
//!    log(1 + z)`, a sum of fixed-point constants in one frame: 256 bits at
//!    2^-214 on the fast leg, 384 bits at 2^-342 on the accurate one, the
//!    former being the top limbs of the latter's tables.
//! 4. **Evaluate.** `log(1 + z) = z·Σ (−1)^k z^k/(k+1)` needs seven terms at
//!    that width, fourteen at the accurate leg's.
//! 5. **Round.** The frame is normalized by its leading zeros; `|log x| ≤ 11357`
//!    and `|log x| ≥ 2^-113` for `x ≠ 1`, so the result is always normal and the
//!    only exact case is `log(1) = +0`.
//!
//! Everything is fixed point in unsigned limbs: an `f128` multiply is soft-float
//! on every target without hardware binary128, so integer limbs are both faster
//! and exactly analyzable.
//!
//! Cancellation is what a logarithm has to survive, and this frame survives it
//! by construction.  A result far below the terms that make it needs `x` near 1,
//! which forces the estimate to either `j = 0` and `e = 0`, where every table
//! term is exactly zero, or `j = 2^18` and `e = −1`, where `LOG0[64]` is the
//! *same rounded constant* as `log_b 2` (for base 2, exactly one) and cancels
//! it exactly.  Both leave `log(1 + z)` alone in the frame with `z` exact, so
//! the accurate leg keeps its full relative accuracy however close `x` comes
//! to 1.
//!
//! Base 2 adds exact cases the natural logarithm never has: `log2(2^k) = k`
//! for every exponent, which the frame delivers by construction — `m = 1`
//! estimates `j = 0`, every table term and `z` are zero, and `k·2^214` is an
//! exact frame value with nothing below its round bit.
//!
//! Base 10's exact cases, `log10(10^k) = k` for `0 ≤ k ≤ 48` (`5^48 < 2^112`
//! keeps `10^k` representable that far), are *not* free: `10^k` is no table
//! reciprocal, so the frame holds `k` plus whatever the tables and the
//! polynomial slipped — under 2^-139 on the fast leg, 2^-245 on the accurate
//! one, against a half-ulp of at least 2^-113 — and the rounder returns `k`
//! by margin rather than by construction.  The fast leg's gate sees those
//! cases at the far ends of the discarded field, nowhere near its tie.

use super::log_tables::{CRUDE, RECIP0, RECIP1, RECIP2};
use super::uint::{
    add_256, add_384, any_below, extract_u128, funnel, funnel_down, leading_zeros_256,
    leading_zeros_384, mhi_approx, mul_hi_64, mul_hi_256, neg_384, shl_256, shl_384, sub_256, wmul,
};
use super::{BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, QUIET_BIT, SIGN_MASK, split};
use super::{log_tables, log2_tables, log10_tables};

/// What a logarithm's base contributes: the constants of the add-back and
/// the polynomial, all in the frames of [`log_tables`].
trait Base {
    /// `log_b 2`, scaled by 2^342: what each unit of the exponent adds.
    const PER_EXPONENT: [u128; 3];
    /// `-log_b(RECIP0[j]/2^31)`, scaled by 2^342.
    const LOG0: &'static [[u128; 3]; 65];
    /// `-log_b(RECIP1[j]/2^31)`, scaled by 2^342.
    const LOG1: &'static [[u128; 3]; 64];
    /// `-log_b(RECIP2[j]/2^31)`, scaled by 2^342.
    const LOG2: &'static [[u128; 3]; 64];
    /// `log_b(e)/(k+1)`, scaled by 2^255: `log_b(1+z)/z = Σ (−1)^k COEF[k] z^k`.
    const COEF: &'static [[u128; 2]; 14];
    /// Half-width of the rounding-tie window the fast leg refuses to decide,
    /// in units of its frame's 2^-214.
    ///
    /// The leg's slip is absolute, not relative: `z` is cut at 2^-145, the
    /// polynomial answers at the same width, and its [`mhi_approx`] products
    /// each fall up to two units short, for under six units of 2^-143 —
    /// 2^73.5 of the frame — of which only `z`'s cut scales with `log_b e`.
    /// Twice that still costs nothing (a normal result's tie window is 2^86
    /// wide at the very floor, 2^112 at the typical magnitude) and leaves the
    /// margin certified in [`ziv_soundness`].
    const ZIV_GATE: u128;
}

/// The natural logarithm's constants.
struct Natural;

impl Base for Natural {
    const PER_EXPONENT: [u128; 3] = log_tables::LN2;
    const LOG0: &'static [[u128; 3]; 65] = &log_tables::LOG0;
    const LOG1: &'static [[u128; 3]; 64] = &log_tables::LOG1;
    const LOG2: &'static [[u128; 3]; 64] = &log_tables::LOG2;
    const COEF: &'static [[u128; 2]; 14] = &log_tables::COEF;
    const ZIV_GATE: u128 = 1 << 75;
}

/// The base-2 logarithm's constants: `log2 e ≈ 1.44` scales the slip, so
/// the gate is one bit wider.
struct Binary;

impl Base for Binary {
    const PER_EXPONENT: [u128; 3] = log2_tables::ONE;
    const LOG0: &'static [[u128; 3]; 65] = &log2_tables::LOG0;
    const LOG1: &'static [[u128; 3]; 64] = &log2_tables::LOG1;
    const LOG2: &'static [[u128; 3]; 64] = &log2_tables::LOG2;
    const COEF: &'static [[u128; 2]; 14] = &log2_tables::COEF;
    const ZIV_GATE: u128 = 1 << 76;
}

/// The base-10 logarithm's constants: `log10 e ≈ 0.43` shrinks only the
/// slip from `z`'s cut — the products' truncations are absolute — so the
/// gate is the natural logarithm's.
struct Decimal;

impl Base for Decimal {
    const PER_EXPONENT: [u128; 3] = log10_tables::LOG10_2;
    const LOG0: &'static [[u128; 3]; 65] = &log10_tables::LOG0;
    const LOG1: &'static [[u128; 3]; 64] = &log10_tables::LOG1;
    const LOG2: &'static [[u128; 3]; 64] = &log10_tables::LOG2;
    const COEF: &'static [[u128; 2]; 14] = &log10_tables::COEF;
    const ZIV_GATE: u128 = 1 << 75;
}

/// The fast leg's floor, as a bound on the frame's high limb: below
/// `|log x| = 2^-16` its absolute slip is worth fewer than fifteen guard bits,
/// so [`accurate`] takes the whole neighbourhood of 1 on its own.
const FAST_FLOOR: u128 = 1 << 70;

/// The exponent field of a result whose frame has no leading zero, biased.
///
/// A frame value with `l` leading zeros is `2^(41 − l)` times its 113-bit
/// significand's leading bit, in either frame: 255 − 214 = 383 − 342 = 41.
const FRAME_EXP: u32 = (BIAS + 41) as u32;

/// The natural logarithm.
#[must_use]
pub fn logq(x: f128) -> f128 {
    log::<Natural>(x)
}

/// The base-2 logarithm.
///
/// Exact at every power of two, `log2(2^k) = k`, down through the subnormals.
#[must_use]
pub fn log2q(x: f128) -> f128 {
    log::<Binary>(x)
}

/// The base-10 logarithm.
///
/// Exact at every representable power of ten: `log10(10^k) = k` for
/// `0 ≤ k ≤ 48`.
#[must_use]
pub fn log10q(x: f128) -> f128 {
    log::<Decimal>(x)
}

/// `log_b x` for the base `B`: the shared fast leg, its gate, and the
/// hand-over to [`accurate`].
#[inline]
fn log<B: Base>(x: f128) -> f128 {
    let bits = x.to_bits();

    // Everything but a positive finite nonzero argument.
    if bits.wrapping_sub(1) >= EXP_MASK - 1 {
        return edge(bits);
    }
    let (m, e) = split(bits);
    let j = crude_log2(m);
    let (high, low) = wmul(m, reciprocal(j));

    // `m·2^(-j/2^18) = 1 + z` scaled by 2^205, exactly, in two's complement.
    let d = [low, high.wrapping_sub(1 << 77)];
    let s = fast::<B>(e, j, ((d[1] << 68) | (low >> 60)) as i128);
    let negative = s[1] >> 127 != 0;
    let magnitude = negate_if(s, negative);

    if magnitude[1] < FAST_FLOOR {
        return accurate::<B>(e, j, d);
    }
    // `2^70 ≤ magnitude[1] < 2^101` (`|log2 x| < 2^14.01`), so its top limb is
    // nonzero and `leading` lands in [27, 57]: every variable shift below
    // stays under 64 bits, one funnel each instead of a `u128` shift pair and
    // its `cmov` guard.
    let leading = ((magnitude[1] >> 64) as u64).leading_zeros();
    let shift = 79 - leading;
    let window = leading - 15;

    // The discarded field scaled by 2^window puts its tie center at the
    // constant 2^127, the gate at `ZIV_GATE << window` — a bare high limb.
    let v = (u128::from(funnel(
        magnitude[0] as u64,
        (magnitude[0] >> 64) as u64,
        window,
    )) << 64)
        | u128::from((magnitude[0] as u64) << window);
    let bound = u128::from(((B::ZIV_GATE >> 64) as u64) << window) << 64;

    if (v ^ (1 << 127)).wrapping_add(bound) <= bound << 1 {
        return accurate::<B>(e, j, d);
    }
    // The gate has already ruled out a tie, so the round bit decides.
    let mantissa = (u128::from(funnel_down(
        magnitude[1] as u64,
        (magnitude[1] >> 64) as u64,
        shift,
    )) << 64)
        | u128::from(funnel_down(
            (magnitude[0] >> 64) as u64,
            magnitude[1] as u64,
            shift,
        ));

    f128::from_bits(
        (u128::from(negative) << 127)
            | ((u128::from(FRAME_EXP - leading) << EXP_SHIFT)
                + (mantissa - IMPLICIT_BIT)
                + u128::from(v >> 127 != 0)),
    )
}

/// `s` or `−s` by a coin-flip sign, in two's complement through an xor mask
/// and a carry-in — never a data-dependent branch.
#[inline]
fn negate_if(s: [u128; 2], negative: bool) -> [u128; 2] {
    let mask = 0_u128.wrapping_sub(u128::from(negative));
    let (low, carry) = (s[0] ^ mask).overflowing_add(mask & 1);

    [low, (s[1] ^ mask).wrapping_add(u128::from(carry))]
}

/// The arguments with no logarithm to compute: zero, negative, and nonfinite.
#[cold]
#[inline(never)]
fn edge(bits: u128) -> f128 {
    if bits & !SIGN_MASK > EXP_MASK {
        return f128::from_bits(bits | QUIET_BIT);
    }
    if bits & !SIGN_MASK == 0 {
        return f128::NEG_INFINITY;
    }
    if bits & SIGN_MASK != 0 {
        return f128::NAN;
    }
    f128::INFINITY
}

/// `⌊2^18·log2(m) + ½⌋` to within one unit, for a significand `m·2^-112`.
///
/// The 8 bits below the leading one pick a bucket; the 55 below that ride the
/// bucket's secant slope.  [`CRUDE`] packs the two as `intercept << 23 | slope`
/// at a fixed 2^-12 of an index step, so the estimate is one multiply wide.
#[inline]
fn crude_log2(m: u128) -> u32 {
    let h = (m >> 49) as u64;
    let entry = CRUDE[(h >> 55) as usize & 255];
    let fit = (entry >> 23) + mul_hi_64(h << 9, entry & ((1 << 23) - 1));

    ((fit + (1 << 11)) >> 12) as u32
}

/// The three table indices of a reduction estimate.
#[inline]
const fn index(j: u32) -> (usize, usize, usize) {
    ((j >> 12) as usize, (j >> 6) as usize & 63, j as usize & 63)
}

/// `2^(-j/2^18)` scaled by 2^93, as the product of the three rounded levels.
#[inline]
fn reciprocal(j: u32) -> u128 {
    let (j0, j1, j2) = index(j);

    u128::from(u64::from(RECIP0[j0]) * u64::from(RECIP1[j1])) * u128::from(RECIP2[j2])
}

/// `log_b(x)` in the fast leg's frame: 256 bits scaled by 2^-214, two's
/// complement.
#[inline]
fn fast<B: Base>(e: i32, j: u32, z: i128) -> [u128; 2] {
    let (j0, j1, j2) = index(j);
    let l = log1p::<B>(z);
    let mut s = scale(e, [B::PER_EXPONENT[1], B::PER_EXPONENT[2]]);

    for t in [&B::LOG0[j0], &B::LOG1[j1], &B::LOG2[j2]] {
        s = add_256(s, [t[1], t[2]]);
    }
    add_256(s, [(l as u128) << 69, (l >> 59) as u128])
}

/// `e·l` for a small signed `e` and an unsigned 256-bit `l`.
#[inline]
fn scale(e: i32, l: [u128; 2]) -> [u128; 2] {
    let a = u128::from(e.unsigned_abs());
    let (high, low) = wmul(a, l[0]);

    negate_if([low, high.wrapping_add(a.wrapping_mul(l[1]))], e < 0)
}

/// `log_b(1 + z)` scaled by 2^145, from `z` at the same scale.
///
/// Only the last product needs all of `z`: a polynomial step answers a
/// correction under 2^-18.7, so the narrower `z·2^128` lands its truncation
/// 2^-147 below the result and saves the whole chain a shift.  The Taylor tail
/// rides on `z^4 < 2^-74` and holds to 2^-138 in 64-bit limbs — a quarter of
/// the multiplier work of the 128-bit steps, which start where that no longer
/// suffices.
#[inline]
fn log1p<B: Base>(z: i128) -> i128 {
    let narrow = z >> 17;
    let short = (z >> 81) as i64;
    let mut tail = coefficient::<B>(6);

    for k in (4..6).rev() {
        tail = coefficient::<B>(k) - mul_hi_i64(short, tail);
    }
    // Balanced instead of Horner: both leaves and the powers of `narrow`
    // overlap each other and the 64-bit tail, so the serial chain is two
    // products shorter.  `n⁴ < 2^-74` at scale 2^128 fits one limb, making
    // the tail's join a single 64-bit product.
    let a = B::COEF[0][1].wrapping_sub(mul_hi_i128(narrow, B::COEF[1][1]) as u128);
    let b = B::COEF[2][1].wrapping_sub(mul_hi_i128(narrow, B::COEF[3][1]) as u128);
    let nn = sqr_hi(narrow);
    let n4 = mhi_approx(nn, nn);
    let q = a
        .wrapping_add(mhi_approx(nn, b))
        .wrapping_add(u128::from(mul_hi_64(n4 as u64, tail as u64)));

    mul_hi_i128(z, q) << 1
}

/// High half of `x²` for a signed `x`, up to two units short.
///
/// Squared through `|x|`, not by a two's complement correction: with `|x| <
/// 2^64` the true high half is nearly zero, and a correction of a product
/// that already fell short would wrap to `2^128 − δ` — which once fed the
/// Horner chain a whole extra `b` term for any `m` within 2^-63 of a table
/// reciprocal (`m = 2 − 2^-77`, say).  `mhi_approx` only ever drops partial
/// products, so on `|x|` it stays nonnegative.
#[inline]
fn sqr_hi(x: i128) -> u128 {
    mhi_approx(x.unsigned_abs(), x.unsigned_abs())
}

/// The `k`-th Taylor coefficient truncated to 64 bits, scaled by 2^-63.
///
/// Only `k ≥ 1` fits: `log2 e` would fill the sign bit, and so would `1/1`.
#[inline]
const fn coefficient<B: Base>(k: usize) -> i64 {
    (B::COEF[k][1] >> 64) as i64
}

/// High half of a signed 64×64-bit product.
#[inline]
const fn mul_hi_i64(x: i64, y: i64) -> i64 {
    ((x as i128 * y as i128) >> 64) as i64
}

/// High half of a signed × unsigned 128×128-bit product, up to two units
/// short — [`mhi_approx`]'s slack, which [`ZIV_GATE`] budgets for.
///
/// The sign correction is an arithmetic mask, not a select: a select on the
/// loop-invariant sign of `z` invites LLVM to clone the whole Horner chain
/// behind a 50/50 branch.
#[inline]
fn mul_hi_i128(x: i128, y: u128) -> i128 {
    mhi_approx(x as u128, y).wrapping_sub(((x >> 127) as u128) & y) as i128
}

/// [`log`] at 384 bits, from the exact reduction the fast leg started from.
///
/// The frame's 2^-342 and the tables' own rounding leave the polynomial as the
/// only real error: 2^-254 relative on `log(1 + z)`, which the worst
/// cancellation this reduction admits — a seventh, where the estimate rounds up
/// from `j = 0.15` — widens to 2^-251, still 137 bits past binary128.
#[cold]
#[inline(never)]
fn accurate<B: Base>(e: i32, j: u32, d: [u128; 2]) -> f128 {
    let (j0, j1, j2) = index(j);
    let mut s = scale_384(e, B::PER_EXPONENT);

    for t in [&B::LOG0[j0], &B::LOG1[j1], &B::LOG2[j2]] {
        s = add_384(s, *t);
    }
    round(add_384(s, log1p_wide::<B>(d)))
}

/// [`scale`] at 384 bits.
fn scale_384(e: i32, l: [u128; 3]) -> [u128; 3] {
    let a = u128::from(e.unsigned_abs());
    let (high, low) = wmul(a, l[0]);
    let (top, middle) = wmul(a, l[1]);
    let (middle, carry) = middle.overflowing_add(high);
    let product = [
        low,
        middle,
        top.wrapping_add(u128::from(carry))
            .wrapping_add(a.wrapping_mul(l[2])),
    ];

    if e < 0 { neg_384(product) } else { product }
}

/// `log_b(1 + z)` in the accurate frame, from the exact `z·2^205`.
///
/// Normalizing `|z|` *before* the product is what keeps the accuracy relative
/// rather than absolute: the frame's own 2^-342 would otherwise be all that is
/// left of an `x` a hair from 1, where the rest of the sum is exactly zero.
fn log1p_wide<B: Base>(d: [u128; 2]) -> [u128; 3] {
    let negative = d[1] >> 127 != 0;
    let z = shl_256(d, 68);
    let mut q = B::COEF[13];

    for c in B::COEF[..13].iter().rev() {
        q = sub_256(*c, shift_right(product(z, q, negative), 17));
    }
    let magnitude = if negative { sub_256([0, 0], d) } else { d };

    if magnitude == [0, 0] {
        return [0; 3];
    }
    let leading = leading_zeros_256(magnitude);
    let w = mul_hi_256(shl_256(magnitude, leading), q);

    // `|z| ≥ 2^-113` keeps the shift within the frame, in either direction.
    let frame = if leading <= 138 {
        shl_384([w[0], w[1], 0], 138 - leading)
    } else {
        let k = leading - 138;
        [(w[0] >> k) | (w[1] << 1 << (127 - k)), w[1] >> k, 0]
    };

    if negative { neg_384(frame) } else { frame }
}

/// High 256 bits of a signed × unsigned 256×256-bit product.
fn product(x: [u128; 2], y: [u128; 2], negative: bool) -> [u128; 2] {
    let high = mul_hi_256(x, y);

    if negative { sub_256(high, y) } else { high }
}

/// `x >> shift` on a signed 256-bit value, for `shift < 128`.
fn shift_right(x: [u128; 2], shift: u32) -> [u128; 2] {
    [
        (x[0] >> shift) | (x[1] << 1 << (127 - shift)),
        ((x[1] as i128) >> shift) as u128,
    ]
}

/// Round a 384-bit frame value scaled by 2^-342 to binary128, ties to even.
fn round(s: [u128; 3]) -> f128 {
    let negative = s[2] >> 127 != 0;
    let magnitude = if negative { neg_384(s) } else { s };

    // `log_b(1) = +0` is the only zero frame; base 2's other exact cases,
    // `log2(2^k) = k`, are integers with nothing below the round bit, and
    // base 10's, `log10(10^k) = k`, carry only the tables' slip there.
    if magnitude == [0; 3] {
        return 0.0;
    }
    let leading = leading_zeros_384(magnitude);
    let shift = 271 - leading;
    let mantissa = extract_u128(magnitude, shift);
    let round_bit = magnitude[(shift as usize - 1) / 128] >> ((shift - 1) % 128) & 1;
    let up = round_bit != 0 && (any_below(magnitude, shift - 1) || mantissa & 1 != 0);

    f128::from_bits(
        (u128::from(negative) << 127)
            | ((u128::from(FRAME_EXP - leading) << EXP_SHIFT)
                + (mantissa - IMPLICIT_BIT)
                + u128::from(up)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_and_special() {
        assert_eq!(logq(1.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(logq(2.0).to_bits(), core::f128::consts::LN_2.to_bits());
        assert_eq!(logq(0.5).to_bits(), (-core::f128::consts::LN_2).to_bits());
        assert_eq!(logq(0.0).to_bits(), f128::NEG_INFINITY.to_bits());
        assert_eq!(logq(-0.0).to_bits(), f128::NEG_INFINITY.to_bits());
        assert_eq!(logq(f128::INFINITY).to_bits(), f128::INFINITY.to_bits());
        assert!(logq(-1.0).is_nan());
        assert!(logq(f128::NEG_INFINITY).is_nan());
        assert!(logq(f128::NAN).is_nan());
        assert!(logq(-f128::NAN).is_nan());
    }

    #[test]
    fn known_values() {
        // ln(10), and the extremes of the range: the least subnormal and MAX.
        assert_eq!(
            logq(10.0).to_bits(),
            0x4000_26bb_1bbb_5551_582d_d4ad_ac57_05a6
        );
        assert_eq!(
            logq(f128::from_bits(1)).to_bits(),
            0xc00c_6546_2822_0780_2c89_d24d_65e9_6274
        );
        assert_eq!(
            logq(f128::MAX).to_bits(),
            0x400c_62e4_2fef_a39e_f357_93c7_6730_07e6
        );
        // One ulp either side of 1, where the accurate leg decides alone and
        // `log(1 + z)` keeps every bit of an exact `z`.
        assert_eq!(
            logq(1.0 + f128::EPSILON).to_bits(),
            0x3f8e_ffff_ffff_ffff_ffff_ffff_ffff_ffff
        );
        assert_eq!(
            logq(1.0 - f128::EPSILON).to_bits(),
            0xbf8f_0000_0000_0000_0000_0000_0000_0001
        );
    }

    /// `m` within 2^-63 of a table reciprocal, where the fast leg's `narrow²`
    /// high limb is nearly zero and must not wrap: `m = 2 − 2^-78` (`j =
    /// 2^18`, `z = −2^-79`), and the subnormal `(2^78 − 1)·2^-16494`.
    #[test]
    fn near_reciprocal() {
        let m = (1_u128 << 112) - (1 << 34);

        assert_eq!(
            logq(f128::from_bits(16383 << 112 | m)).to_bits(),
            0x3ffe_62e4_2fef_a39e_f357_93c3_6730_07e6
        );
        assert_eq!(
            log2q(f128::from_bits(16383 << 112 | m)).to_bits(),
            0x3ffe_ffff_ffff_ffff_ffff_fffa_3aae_26b5
        );
        assert_eq!(
            log2q(f128::from_bits((1 << 78) - 1)).to_bits(),
            0xc00d_0080_0000_0000_0000_0000_0017_1547
        );
        assert_eq!(
            log10q(f128::from_bits(16383 << 112 | m)).to_bits(),
            0x3ffd_3441_3509_f79f_ef31_1f0f_39e8_b454
        );
        assert_eq!(
            log10q(f128::from_bits((1 << 78) - 1)).to_bits(),
            0xc00b_34db_55a4_7c9b_bf28_b7a2_3ccd_8e80
        );
    }

    #[test]
    fn log2_exact_and_special() {
        assert_eq!(log2q(1.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(log2q(0.0).to_bits(), f128::NEG_INFINITY.to_bits());
        assert_eq!(log2q(f128::INFINITY).to_bits(), f128::INFINITY.to_bits());
        assert!(log2q(-1.0).is_nan());
        assert!(log2q(f128::NAN).is_nan());
        // Every power of two, subnormals included, is exact.
        for k in -16494..=16383_i32 {
            let x = super::super::ldexp(1.0, k);
            assert_eq!(log2q(x).to_bits(), (k as f128).to_bits(), "log2(2^{k})");
        }
    }

    #[test]
    fn log2_known_values() {
        // log2(10), log2(e), and one ulp either side of 1.
        assert_eq!(log2q(10.0).to_bits(), core::f128::consts::LOG2_10.to_bits());
        assert_eq!(
            log2q(core::f128::consts::E).to_bits(),
            0x3fff_7154_7652_b82f_e177_7d0f_fda0_d23a
        );
        assert_eq!(
            log2q(1.0 + f128::EPSILON).to_bits(),
            0x3f8f_7154_7652_b82f_e177_7d0f_fda0_d23a
        );
        assert_eq!(
            log2q(1.0 - f128::EPSILON).to_bits(),
            0xbf8f_7154_7652_b82f_e177_7d0f_fda0_d23b
        );
    }

    #[test]
    fn log10_exact_and_special() {
        assert_eq!(log10q(1.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(log10q(0.0).to_bits(), f128::NEG_INFINITY.to_bits());
        assert_eq!(log10q(f128::INFINITY).to_bits(), f128::INFINITY.to_bits());
        assert!(log10q(-1.0).is_nan());
        assert!(log10q(f128::NAN).is_nan());
        // Every representable power of ten is exact: `5^48 < 2^112 < 5^49`.
        let mut x = 1.0_f128;
        for k in 0..=48 {
            assert_eq!(log10q(x).to_bits(), (k as f128).to_bits(), "log10(10^{k})");
            x *= 10.0;
        }
    }

    #[test]
    fn log10_known_values() {
        // log10(2), log10(e), the extremes of the range, and one ulp either
        // side of 1.
        assert_eq!(log10q(2.0).to_bits(), core::f128::consts::LOG10_2.to_bits());
        assert_eq!(
            log10q(core::f128::consts::E).to_bits(),
            0x3ffd_bcb7_b152_6e50_e32a_6ab7_555f_5a67
        );
        assert_eq!(
            log10q(f128::from_bits(1)).to_bits(),
            0xc00b_3653_051d_20c1_8a14_3b80_1b7c_5661
        );
        assert_eq!(
            log10q(f128::MAX).to_bits(),
            0x400b_3441_3509_f79f_ef31_1f12_b358_16f9
        );
        assert_eq!(
            log10q(1.0 + f128::EPSILON).to_bits(),
            0x3f8d_bcb7_b152_6e50_e32a_6ab7_555f_5a67
        );
        assert_eq!(
            log10q(1.0 - f128::EPSILON).to_bits(),
            0xbf8d_bcb7_b152_6e50_e32a_6ab7_555f_5a69
        );
    }
}

/// MPFR certification that [`ZIV_GATE`] covers the fast leg's true error with
/// the 2× margin the project requires.  Run with
/// `CC=clang cargo +nightly test --release --features "f128 mpfr"`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::super::MANTISSA_MASK;
    use super::*;
    use rug::{Float, ops::Pow};

    const PRECISION: u32 = 400;
    const SAMPLES: u64 = 200_000;

    fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A random positive significand at an exponent uniform over every binade,
    /// subnormals included — or, every eighth draw, a significand a few ulps
    /// from a table reciprocal `2^(j/2^18)`, where the reduced `z` is tiny.
    fn sample(i: u64) -> f128 {
        let bits = u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64;
        let exponent = (bits >> 120) % 0x7fff;

        if i % 8 == 7 {
            let j = (bits & 0x3ffff) as u32;
            let m = super::super::exp2q(f128::from(j) / 262_144.0).to_bits() & MANTISSA_MASK;
            let d = ((bits >> 20) % 9) as i128 - 4;
            return f128::from_bits(((exponent.max(1) << EXP_SHIFT | m) as i128 + d) as u128);
        }
        f128::from_bits(exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// The fast leg's frame value, as [`log`] sees it.
    fn leg<B: Base>(x: f128) -> [u128; 2] {
        let (m, e) = split(x.to_bits());
        let j = crude_log2(m);
        let (high, low) = wmul(m, reciprocal(j));
        let d = [low, high.wrapping_sub(1 << 77)];

        fast::<B>(e, j, ((d[1] << 68) | (low >> 60)) as i128)
    }

    /// A two's complement frame value as an exact MPFR integer.
    fn value(s: [u128; 2]) -> Float {
        let negative = s[1] >> 127 != 0;
        let magnitude = if negative { sub_256([0, 0], s) } else { s };
        let high =
            Float::with_val(PRECISION, magnitude[1]) * Float::with_val(PRECISION, 2).pow(128);
        let sum: Float = high + Float::with_val(PRECISION, magnitude[0]);

        if negative { -sum } else { sum }
    }

    /// Worst `|leg − log_b x| / ZIV_GATE` over the sample, in frame units —
    /// which is exactly what the gate compares, the slip being absolute.
    fn certify<B: Base>(name: &str, truth: fn(Float) -> Float) {
        let unit: Float = Float::with_val(PRECISION, 2).pow(-214);
        let mut worst = 0.0;
        let mut worst_x = 0.0;

        for i in 0..SAMPLES {
            let x = sample(i);
            let s = leg::<B>(x);
            let negative = s[1] >> 127 != 0;
            let magnitude = if negative { sub_256([0, 0], s) } else { s };

            // The leg answers only above its floor; below it the accurate leg
            // decides alone.
            if x == 0.0 || magnitude[1] < FAST_FLOOR {
                continue;
            }
            let truth = truth(Float::with_val(PRECISION, x)) / unit.clone();
            let ratio =
                Float::with_val(PRECISION, truth - value(s)).abs().to_f64() / B::ZIV_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        println!("{name} fast leg: worst |err|/gate = {worst:.4} at x={worst_x:?}");
        assert!(
            worst < 0.5,
            "{name} gate covers only {:.2}× the slip at x={worst_x:?}",
            1.0 / worst
        );
    }

    #[test]
    fn fast_leg_is_sound() {
        certify::<Natural>("logq", Float::ln);
    }

    #[test]
    fn log2_fast_leg_is_sound() {
        certify::<Binary>("log2q", Float::log2);
    }

    #[test]
    fn log10_fast_leg_is_sound() {
        certify::<Decimal>("log10q", Float::log10);
    }
}
