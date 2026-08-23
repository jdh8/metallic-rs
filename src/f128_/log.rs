//! The binary128 natural logarithm.
//!
//! The reduction is done in *log space*, so the table of logarithms to add back
//! is the only table the sum needs:
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
//! *same rounded constant* as [`LN2`] and cancels it exactly.  Both leave
//! `log(1 + z)` alone in the frame with `z` exact, so the accurate leg keeps its
//! full relative accuracy however close `x` comes to 1.

use super::log_tables::{COEF, CRUDE, LN2, LOG0, LOG1, LOG2, RECIP0, RECIP1, RECIP2};
use super::uint::{
    add_256, add_384, extract_u128, leading_zeros_256, leading_zeros_384, mhi, mul_hi_64,
    mul_hi_256, neg_384, shl_256, shl_384, sub_256, wmul,
};
use super::{BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, QUIET_BIT, SIGN_MASK, split};

/// Half-width of the rounding-tie window the fast leg refuses to decide, in
/// units of its frame's 2^-214.
///
/// The leg's slip is absolute, not relative: `z` is cut at 2^-145 and the
/// polynomial answers at the same width, for at most four units of it — 2^-143,
/// or 2^71 of the frame.  Four times that costs nothing (a normal result's tie
/// window is 2^86 wide at the very floor, 2^112 at the typical magnitude) and
/// leaves the margin certified in [`ziv_soundness`].
const ZIV_GATE: u128 = 1 << 74;

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
    let s = fast(e, j, ((d[1] << 68) | (low >> 60)) as i128);
    let negative = s[1] >> 127 != 0;
    let magnitude = if negative { sub_256([0, 0], s) } else { s };

    if magnitude[1] < FAST_FLOOR {
        return accurate(e, j, d);
    }
    let leading = magnitude[1].leading_zeros();
    let shift = 143 - leading;
    let rest = magnitude[0] & (u128::MAX >> (128 - shift));
    let half = 1 << (shift - 1);

    if rest.abs_diff(half) <= ZIV_GATE {
        return accurate(e, j, d);
    }
    // The gate has already ruled out a tie, so the round bit decides.
    let mantissa = (magnitude[0] >> shift) | (magnitude[1] << (128 - shift));

    f128::from_bits(
        (u128::from(negative) << 127)
            | ((u128::from(FRAME_EXP - leading) << EXP_SHIFT)
                + (mantissa - IMPLICIT_BIT)
                + u128::from(rest > half)),
    )
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

/// `log(x)` in the fast leg's frame: 256 bits scaled by 2^-214, two's
/// complement.
#[inline]
fn fast(e: i32, j: u32, z: i128) -> [u128; 2] {
    let (j0, j1, j2) = index(j);
    let l = log1p(z);
    let mut s = scale(e, [LN2[1], LN2[2]]);

    for t in [&LOG0[j0], &LOG1[j1], &LOG2[j2]] {
        s = add_256(s, [t[1], t[2]]);
    }
    add_256(s, [(l as u128) << 69, (l >> 59) as u128])
}

/// `e·l` for a small signed `e` and an unsigned 256-bit `l`.
#[inline]
fn scale(e: i32, l: [u128; 2]) -> [u128; 2] {
    let a = u128::from(e.unsigned_abs());
    let (high, low) = wmul(a, l[0]);
    let product = [low, high.wrapping_add(a.wrapping_mul(l[1]))];

    if e < 0 {
        sub_256([0, 0], product)
    } else {
        product
    }
}

/// `log(1 + z)` scaled by 2^145, from `z` at the same scale.
///
/// Only the last product needs all of `z`: a Horner step answers a correction
/// under 2^-18.7, so the narrower `z·2^128` lands its truncation 2^-147 below
/// the result and saves the whole chain a shift.  The Taylor tail rides on
/// `z^4 < 2^-74` and holds to 2^-138 in 64-bit limbs — a quarter of the
/// multiplier work of the 128-bit steps, which start where that no longer
/// suffices.
#[inline]
fn log1p(z: i128) -> i128 {
    let narrow = z >> 17;
    let short = (z >> 81) as i64;
    let mut tail = coefficient(6);

    for k in (4..6).rev() {
        tail = coefficient(k) - mul_hi_i64(short, tail);
    }
    let mut q = COEF[3][1].wrapping_sub(mul_hi_i128(narrow, (tail as u128) << 64) as u128);

    for c in COEF[..3].iter().rev() {
        q = c[1].wrapping_sub(mul_hi_i128(narrow, q) as u128);
    }
    mul_hi_i128(z, q) << 1
}

/// The `k`-th Taylor coefficient truncated to 64 bits, scaled by 2^-63.
///
/// Only `k ≥ 1` fits: `1/1` would fill the sign bit.
#[inline]
const fn coefficient(k: usize) -> i64 {
    (COEF[k][1] >> 64) as i64
}

/// High half of a signed 64×64-bit product.
#[inline]
const fn mul_hi_i64(x: i64, y: i64) -> i64 {
    ((x as i128 * y as i128) >> 64) as i64
}

/// High half of a signed × unsigned 128×128-bit product.
#[inline]
fn mul_hi_i128(x: i128, y: u128) -> i128 {
    mhi(x as u128, y).wrapping_sub(if x < 0 { y } else { 0 }) as i128
}

/// [`logq`] at 384 bits, from the exact reduction the fast leg started from.
///
/// The frame's 2^-342 and the tables' own rounding leave the polynomial as the
/// only real error: 2^-254 relative on `log(1 + z)`, which the worst
/// cancellation this reduction admits — a seventh, where the estimate rounds up
/// from `j = 0.15` — widens to 2^-251, still 137 bits past binary128.
#[cold]
#[inline(never)]
fn accurate(e: i32, j: u32, d: [u128; 2]) -> f128 {
    let (j0, j1, j2) = index(j);
    let mut s = scale_384(e, LN2);

    for t in [&LOG0[j0], &LOG1[j1], &LOG2[j2]] {
        s = add_384(s, *t);
    }
    round(add_384(s, log1p_wide(d)))
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

/// `log(1 + z)` in the accurate frame, from the exact `z·2^205`.
///
/// Normalizing `|z|` *before* the product is what keeps the accuracy relative
/// rather than absolute: the frame's own 2^-342 would otherwise be all that is
/// left of an `x` a hair from 1, where the rest of the sum is exactly zero.
fn log1p_wide(d: [u128; 2]) -> [u128; 3] {
    let negative = d[1] >> 127 != 0;
    let z = shl_256(d, 68);
    let mut q = COEF[13];

    for c in COEF[..13].iter().rev() {
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

    // `log(1) = +0` is the only exact case, and the only zero frame.
    if magnitude == [0; 3] {
        return 0.0;
    }
    let leading = leading_zeros_384(magnitude);
    let shift = 271 - leading;
    let mantissa = extract_u128(magnitude, shift);
    let round_bit = magnitude[(shift as usize - 1) / 128] >> ((shift - 1) % 128) & 1;
    let up = round_bit != 0 && (below(magnitude, shift - 1) || mantissa & 1 != 0);

    f128::from_bits(
        (u128::from(negative) << 127)
            | ((u128::from(FRAME_EXP - leading) << EXP_SHIFT)
                + (mantissa - IMPLICIT_BIT)
                + u128::from(up)),
    )
}

/// Whether any of the low `n` bits of a 384-bit value is set.
fn below(x: [u128; 3], n: u32) -> bool {
    let word = (n / 128) as usize;
    let bits = n % 128;

    x[..word].iter().any(|&limb| limb != 0)
        || (bits != 0 && x[word] & (u128::MAX >> (128 - bits)) != 0)
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
    /// subnormals included.
    fn sample(i: u64) -> f128 {
        let bits = u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64;
        let exponent = (bits >> 120) % 0x7fff;

        f128::from_bits(exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// The fast leg's frame value, as [`logq`] sees it.
    fn leg(x: f128) -> [u128; 2] {
        let (m, e) = split(x.to_bits());
        let j = crude_log2(m);
        let (high, low) = wmul(m, reciprocal(j));
        let d = [low, high.wrapping_sub(1 << 77)];

        fast(e, j, ((d[1] << 68) | (low >> 60)) as i128)
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

    /// Worst `|leg − log x| / ZIV_GATE` over the sample, in frame units — which
    /// is exactly what the gate compares, the slip being absolute.
    #[test]
    fn fast_leg_is_sound() {
        let unit: Float = Float::with_val(PRECISION, 2).pow(-214);
        let mut worst = 0.0;
        let mut worst_x = 0.0;

        for i in 0..SAMPLES {
            let x = sample(i);
            let s = leg(x);
            let negative = s[1] >> 127 != 0;
            let magnitude = if negative { sub_256([0, 0], s) } else { s };

            // The leg answers only above its floor; below it the accurate leg
            // decides alone.
            if x == 0.0 || magnitude[1] < FAST_FLOOR {
                continue;
            }
            let truth = Float::with_val(PRECISION, x).ln() / unit.clone();
            let ratio =
                Float::with_val(PRECISION, truth - value(s)).abs().to_f64() / ZIV_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        println!("logq fast leg: worst |err|/gate = {worst:.4} at x={worst_x:?}");
        assert!(
            worst < 0.5,
            "logq gate covers only {:.2}× the slip at x={worst_x:?}",
            1.0 / worst
        );
    }
}
