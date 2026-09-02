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
//!
//! [`log1pq`] is the natural logarithm behind a different front end.  Below
//! `|x| = 2^-18` the argument is its own reduced `z`, exact, and the Taylor
//! ratio `log(1 + z)/z` multiplies the input significand in floating form —
//! full relative accuracy down to `|x| = 2^-113`, below which `x` itself is
//! the correctly rounded answer.  Above it `1 + x` is formed *exactly* in a
//! 256-bit significand (`113 − e` bits below 1, `max(113, e + 1)` above, so
//! only `x ≥ 2^256` drops its 1, a relative slip under 2^-256), its 384-bit
//! product with the reciprocal carries `z` to 2^-333 for the accurate leg,
//! and the fast leg, its gate, and the rounder are [`logq`]'s unchanged.

use super::log_tables::{CRUDE, RECIP0, RECIP1, RECIP2};
use super::uint::{
    add_256, add_384, any_below, extract_u128, funnel, funnel_down, leading_zeros_256,
    leading_zeros_384, mhi_approx, mul_hi_64, mul_hi_256, neg_384, shl_256, shl_384, shr_256_sat,
    sub_256, wmul,
};
use super::{BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, MANTISSA_MASK, QUIET_BIT, SIGN_MASK, split};
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

/// [`log1pq`]'s floor: its general leg starts at `|x| = 2^-18`, where
/// `|log(1 + x)| > 2^-19`, so this is never crossed and only pins the shift
/// windows of [`finish`].
const FLOOR_1P: u128 = 1 << 67;

/// The bit pattern of 1.
const ONE: u128 = (BIAS as u128) << EXP_SHIFT;

/// Magnitudes below 2^-18, where [`log1pq`]'s argument is its own reduction.
const SMALL: u128 = ((BIAS - 18) as u128) << EXP_SHIFT;

/// Magnitudes below 2^-113, where `log(1 + x)` rounds to `x`.
const TINY: u128 = ((BIAS - 113) as u128) << EXP_SHIFT;

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

/// `log(1 + x)`, without rounding `1 + x` first.
///
/// `log1p(±0) = ±0`, and below `|x| = 2^-113` the result is `x` itself.
#[must_use]
pub fn log1pq(x: f128) -> f128 {
    let bits = x.to_bits();
    let magnitude = bits & !SIGN_MASK;

    if magnitude < SMALL {
        return small(bits);
    }
    // Nonfinite, or `x ≤ −1`.
    if magnitude >= EXP_MASK || bits >= SIGN_MASK | ONE {
        return edge1p(bits);
    }
    let (e, j, d) = reduce1p(bits);

    finish::<Natural>(e, j, d, FLOOR_1P)
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

    // `m·2^(-j/2^18) = 1 + z` scaled by 2^205, exactly, in two's complement:
    // the frame's 2^333 is the same two limbs one limb up.
    finish::<B>(e, j, [0, low, high.wrapping_sub(1 << 77)], FAST_FLOOR)
}

/// The fast leg on a reduction `z` at 2^333 in two's complement — exact to
/// 2^-205 at least, which is all [`accurate`] needs, and cut at 2^-145 for
/// the leg — then its gate and the rounding.
#[inline]
fn finish<B: Base>(e: i32, j: u32, d: [u128; 3], floor: u128) -> f128 {
    let s = fast::<B>(e, j, z_fast(d));
    let negative = s[1] >> 127 != 0;
    let magnitude = negate_if(s, negative);

    if magnitude[1] < floor {
        return accurate::<B>(e, j, d);
    }
    // `2^67 ≤ floor ≤ magnitude[1] < 2^101` (`|log2 x| < 2^14.01`), so its top
    // limb is nonzero and `leading` lands in [27, 60]: every variable shift
    // below stays under 64 bits, one funnel each instead of a `u128` shift
    // pair and its `cmov` guard.
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

/// The fast leg's `z` at 2^145 from the frame's at 2^333.
#[inline]
const fn z_fast(d: [u128; 3]) -> i128 {
    ((d[2] << 68) | (d[1] >> 60)) as i128
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

/// The arguments with no `log(1 + x)` to compute: nonfinite, and `x ≤ −1`.
#[cold]
#[inline(never)]
fn edge1p(bits: u128) -> f128 {
    if bits & !SIGN_MASK > EXP_MASK {
        return f128::from_bits(bits | QUIET_BIT);
    }
    if bits == SIGN_MASK | ONE {
        return f128::NEG_INFINITY;
    }
    if bits & SIGN_MASK != 0 {
        return f128::NAN;
    }
    f128::INFINITY
}

/// The reduction of the exact `1 + x` for `x ≥ 2^-18` and `−1 < x ≤ −2^-18`:
/// the exponent, the estimate, and `z` at 2^333 in the frame's two's
/// complement.
#[inline]
fn reduce1p(bits: u128) -> (i32, u32, [u128; 3]) {
    let (m, e) = split(bits & !SIGN_MASK);
    let (big, e) = one_plus(m, e, bits >> 127 != 0);
    let j = crude_log2_top((big[1] >> 64) as u64);
    let r = reciprocal(j);

    // `big·r = (1 + z)·2^348` exactly, 256 bits by 93; the frame keeps 2^333
    // of it, dropping fifteen bits under the accurate leg's 2^-342 — nothing
    // against a result of 2^-19 or more, and the tiny `z` that would care
    // are the fast leg's, cut at 2^-145 anyway.
    let (h1, l1) = wmul(big[1], r);
    let (h0, l0) = wmul(big[0], r);
    let (middle, carry) = l1.overflowing_add(h0);
    let top = h1.wrapping_add(u128::from(carry)).wrapping_sub(1 << 92);

    (
        e,
        j,
        [
            (l0 >> 15) | (middle << 113),
            (middle >> 15) | (top << 113),
            ((top as i128) >> 15) as u128,
        ],
    )
}

/// `1 + x` as an exact 256-bit significand in `[2^255, 2^256)` with its
/// exponent, from `|x| = m·2^(e − 112)`, `e ≥ −18`.
///
/// Below 1 the sum needs `113 − e` bits and above it `max(113, e + 1)`, so
/// only `x ≥ 2^256` loses its 1: a relative slip under 2^-256.
#[inline]
fn one_plus(m: u128, e: i32, negative: bool) -> ([u128; 2], i32) {
    if e < 0 {
        let t = shl_256([m, 0], (143 + e) as u32);

        if negative {
            // `1 − |x| ≥ 2^-113`: at most 113 leading zeros, none lost.
            let s = sub_256([0, 1 << 127], t);
            let leading = leading_zeros_256(s);
            return (shl_256(s, leading), -(leading as i32));
        }
        return (add_256([0, 1 << 127], t), 0);
    }
    let one = if e < 256 {
        shl_256([1, 0], (255 - e) as u32)
    } else {
        [0; 2]
    };
    let (high, overflow) = (m << 15).overflowing_add(one[1]);

    // A carry out is halved exactly: the bottom bit is set only at `e = 255`,
    // where `m·2^143 + 1` cannot carry.
    if overflow {
        (
            [(one[0] >> 1) | (high << 127), (high >> 1) | (1 << 127)],
            e + 1,
        )
    } else {
        ([one[0], high], e)
    }
}

/// `log(1 + x)` for `|x| < 2^-18`: the argument is its own reduced `z`, and
/// the Taylor ratio `log(1 + z)/z` multiplies the input significand in
/// floating form, keeping full relative accuracy down to `|x| = 2^-113`.
/// Below that `x − x²/2 + ⋯` rounds to `x` itself: at `|x| = 2^-113` the
/// square lands exactly on the tie and the cube breaks it toward `x`.
#[inline]
fn small(bits: u128) -> f128 {
    let magnitude = bits & !SIGN_MASK;

    if magnitude < TINY {
        return f128::from_bits(bits);
    }
    let m = magnitude & MANTISSA_MASK | IMPLICIT_BIT;
    let e = (magnitude >> EXP_SHIFT) as i32 - BIAS;
    let negative = bits >> 127 != 0;
    let (high, low) = small_leg(m, e, negative);

    // `|log(1 + x)|·2^(254 − e)` leads within two bits of 2^255.
    let leading = high.leading_zeros();
    let n = shl_256([low, high], leading);
    // The top 128 of the 143 discarded bits, tie center at 2^127.
    let v = (n[1] << 113) | (n[0] >> 15);

    if (v ^ (1 << 127)).wrapping_add(SMALL_GATE) <= SMALL_GATE << 1 {
        return small_accurate(m, e, negative);
    }
    f128::from_bits(
        (bits & SIGN_MASK)
            | ((((e + 1 - leading as i32 + BIAS) as u128) << EXP_SHIFT)
                + ((n[1] >> 15) - IMPLICIT_BIT)
                + u128::from(v >> 127 != 0)),
    )
}

/// The fast leg of [`small`]: `|log(1 + x)|·2^(254 − e)` as `(high, low)`,
/// the exact product of the input significand and [`ratio`] at 2^127.  The
/// ratio's slip — a few units of 2^-128 from its truncated products, plus
/// the `x⁷/8 < 2^-129` past its last term — is all the error there is.
#[inline]
fn small_leg(m: u128, e: i32, negative: bool) -> (u128, u128) {
    // `z·2^145 = ±m·2^(e + 33)`, cut at 2^-145 below `e = −33`, which moves
    // the ratio by half that.
    let z = if e >= -33 {
        m << (e + 33)
    } else {
        m >> (-33 - e)
    };
    let mask = 0_u128.wrapping_sub(u128::from(negative));

    wmul(
        m << 15,
        ratio::<Natural>((z ^ mask).wrapping_sub(mask) as i128),
    )
}

/// Half-width of the tie window [`small`]'s fast leg refuses to decide, in
/// units of its 128-bit discarded field: the ratio's slip is under 2^-125
/// relative, 2^116 of the field, and [`ziv_soundness`] certifies the margin.
const SMALL_GATE: u128 = 1 << 118;

/// [`small`] at 256 bits, on the exact `z = x`.
#[cold]
#[inline(never)]
fn small_accurate(m: u128, e: i32, negative: bool) -> f128 {
    // `z·2^273 = ±m·2^(e + 161)`, exact since `e ≥ −113`.
    let z = shl_256([m, 0], (e + 161) as u32);
    let z = if negative { sub_256([0, 0], z) } else { z };
    let w = log1p_wide::<Natural>(z, [0, m << 15], negative);
    let leading = w[1].leading_zeros();
    let n = shl_256(w, leading);
    let mantissa = n[1] >> 15;
    let round_bit = n[1] >> 14 & 1 != 0;
    let sticky = n[1] & ((1 << 14) - 1) != 0 || n[0] != 0;
    let up = round_bit && (sticky || mantissa & 1 != 0);

    f128::from_bits(
        (u128::from(negative) << 127)
            | ((((e + 1 - leading as i32 + BIAS) as u128) << EXP_SHIFT)
                + (mantissa - IMPLICIT_BIT)
                + u128::from(up)),
    )
}

/// `⌊2^18·log2(m) + ½⌋` to within one unit, for a significand `m·2^-112`.
#[inline]
const fn crude_log2(m: u128) -> u32 {
    crude_log2_top((m >> 49) as u64)
}

/// [`crude_log2`] from the top 64 bits of the significand, leading bit at 63.
///
/// The 8 bits below the leading one pick a bucket; the 55 below that ride the
/// bucket's secant slope.  [`CRUDE`] packs the two as `intercept << 23 | slope`
/// at a fixed 2^-12 of an index step, so the estimate is one multiply wide.
#[inline]
const fn crude_log2_top(h: u64) -> u32 {
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
/// Only this last product needs all of `z`: [`ratio`] answers a correction
/// under 2^-18.7 from the narrower `z·2^128`.
#[inline]
fn log1p<B: Base>(z: i128) -> i128 {
    mul_hi_i128(z, ratio::<B>(z)) << 1
}

/// `log_b(1 + z)/z` scaled by 2^127, from `z` at 2^145, `|z| < 2^-18`.
///
/// A polynomial step answers a correction under 2^-18, so the narrower
/// `z·2^128` lands its truncation 2^-146 below the result and saves the
/// whole chain a shift.  The Taylor tail rides on `z^4 < 2^-72` and holds to
/// 2^-136 in 64-bit limbs — a quarter of the multiplier work of the 128-bit
/// steps, which start where that no longer suffices.
#[inline]
fn ratio<B: Base>(z: i128) -> u128 {
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

    a.wrapping_add(mhi_approx(nn, b))
        .wrapping_add(u128::from(mul_hi_64(n4 as u64, tail as u64)))
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
fn accurate<B: Base>(e: i32, j: u32, d: [u128; 3]) -> f128 {
    let (j0, j1, j2) = index(j);
    let mut s = scale_384(e, B::PER_EXPONENT);

    for t in [&B::LOG0[j0], &B::LOG1[j1], &B::LOG2[j2]] {
        s = add_384(s, *t);
    }
    let negative = d[2] >> 127 != 0;
    let magnitude = if negative { neg_384(d) } else { d };

    if magnitude == [0; 3] {
        return round(s);
    }
    // Normalizing `|z|` *before* the product is what keeps the accuracy
    // relative rather than absolute: the frame's own 2^-342 would otherwise be
    // all that is left of an `x` a hair from 1, where the rest of the sum is
    // exactly zero.
    let leading = leading_zeros_384(magnitude);
    let top = shl_384(magnitude, leading);
    let z = [(d[0] >> 60) | (d[1] << 68), (d[1] >> 60) | (d[2] << 68)];
    let w = log1p_wide::<B>(z, [top[1], top[2]], negative);

    // `w` is `|log_b(1 + z)|` at 2^(204 + leading); the frame is at 2^342.
    let frame = if leading <= 138 {
        shl_384([w[0], w[1], 0], 138 - leading)
    } else {
        let [low, high] = shr_256_sat(w, leading - 138);
        [low, high, 0]
    };
    round(add_384(s, if negative { neg_384(frame) } else { frame }))
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

/// `|log_b(1 + z)|` at 256 bits from `z·2^273` in two's complement, for the
/// polynomial, and `|z|` normalized to `[2^255, 2^256)`, for the product:
/// the result rides the normalized scale, halved — `|z|·2^S` in,
/// `|log_b(1 + z)|·2^(S − 1)` out, up to two units short.
fn log1p_wide<B: Base>(z: [u128; 2], n: [u128; 2], negative: bool) -> [u128; 2] {
    let mut q = B::COEF[13];

    for c in B::COEF[..13].iter().rev() {
        q = sub_256(*c, shift_right(product(z, q, negative), 17));
    }
    mul_hi_256(n, q)
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

    /// MPFR answers at the seams of [`log1pq`], as bit patterns: the two
    /// sides of the 2^-18 hand-over, the least `1 + x`, the largest `x`
    /// below 1, the last `x` whose 1 is kept (2^255) and the first whose 1 is
    /// dropped (2^256), and a deep floating-leg pair.
    const KNOWN_1P: [(u128, u128); 10] = [
        (
            0x3fec_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
            0x3fec_ffff_c000_0aaa_a8aa_ab11_10fb_bbbf,
        ),
        (
            0xbfec_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
            0xbfed_0000_2000_0555_5655_5588_8893_3335,
        ),
        (
            0x3fed_0000_0000_0000_0000_0000_0000_0000,
            0x3fec_ffff_c000_0aaa_a8aa_ab11_10fb_bbc0,
        ),
        (
            0xbfed_0000_0000_0000_0000_0000_0000_0000,
            0xbfed_0000_2000_0555_5655_5588_8893_3335,
        ),
        (
            0xbffe_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
            0xc005_394d_7251_8e72_52d3_5076_0918_66f9,
        ),
        (
            0x3ffe_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
            0x3ffe_62e4_2fef_a39e_f357_93c7_6730_07e5,
        ),
        (
            0x40fe_0000_0000_0000_0000_0000_0000_0000,
            0x4006_6181_4bbf_b3fb_5464_3c33_9fc8_d7de,
        ),
        (
            0x40ff_0000_0000_0000_0000_0000_0000_0000,
            0x4006_62e4_2fef_a39e_f357_93c7_6730_07e6,
        ),
        (
            0x3f9b_0000_0000_0000_0000_0000_0000_0000,
            0x3f9a_ffff_ffff_ffff_ffff_ffff_ffff_f000,
        ),
        (
            0xbf9b_0000_0000_0000_0000_0000_0000_0000,
            0xbf9b_0000_0000_0000_0000_0000_0000_0800,
        ),
    ];

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
    fn log1p_exact_and_special() {
        use super::super::ldexp;

        assert_eq!(log1pq(0.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(log1pq(-0.0).to_bits(), (-0.0_f128).to_bits());
        assert_eq!(log1pq(-1.0).to_bits(), f128::NEG_INFINITY.to_bits());
        assert_eq!(log1pq(f128::INFINITY).to_bits(), f128::INFINITY.to_bits());
        assert!(log1pq(-1.5).is_nan());
        assert!(log1pq(f128::NEG_INFINITY).is_nan());
        assert!(log1pq(f128::NAN).is_nan());
        assert!(log1pq(-f128::NAN).is_nan());
        // `1 + x` a power of two: the reduction is exactly zero.
        assert_eq!(log1pq(1.0).to_bits(), core::f128::consts::LN_2.to_bits());
        assert_eq!(
            log1pq(3.0).to_bits(),
            (2.0 * core::f128::consts::LN_2).to_bits()
        );
        assert_eq!(
            log1pq(-0.5).to_bits(),
            (-core::f128::consts::LN_2).to_bits()
        );
        assert_eq!(
            log1pq(-0.75).to_bits(),
            (-2.0 * core::f128::consts::LN_2).to_bits()
        );
        // Below 2^-113 the argument is the answer, subnormals included; at
        // 2^-113 the square is a quarter ulp and still rounds away.
        for x in [
            f128::from_bits(1),
            f128::MIN_POSITIVE,
            ldexp(1.0, -114),
            ldexp(-1.0, -114),
            ldexp(1.0, -113),
            ldexp(-1.0, -113),
        ] {
            assert_eq!(log1pq(x).to_bits(), x.to_bits(), "log1p({x:?})");
        }
        // At 2^-112 the square is exactly half an ulp: the cube decides.
        let x = ldexp(1.0, -112);
        assert_eq!(log1pq(x).to_bits(), x.to_bits() - 1);
        assert_eq!(log1pq(-x).to_bits(), (-x).to_bits() + 1);
    }

    #[test]
    fn log1p_known_values() {
        for (x, want) in KNOWN_1P {
            assert_eq!(log1pq(f128::from_bits(x)).to_bits(), want, "log1p({x:#x})");
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

    /// [`log1pq`]'s general band: the exponent uniform over `[−18, 16383]`
    /// (positive) or `[−18, −1]` (negative) — or, every eighth draw, `1 + x`
    /// within a few ulps of a table reciprocal `2^(E + j/2^18)`.
    fn sample_1p(i: u64) -> f128 {
        let bits = u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64;
        let span = if bits >> 127 != 0 { 18 } else { 16402 };
        let e = ((bits >> 112 & 0x7fff) % span) as i32 - 18;

        if i % 8 == 7 {
            let j = (bits & 0x3ffff) as u32;
            let y = super::super::exp2q(f128::from(j) / 262_144.0 + e.clamp(-17, 40) as f128);
            let d = ((bits >> 20) % 9) as i128 - 4;
            return f128::from_bits(((y - 1.0).to_bits() as i128 + d) as u128);
        }
        f128::from_bits(bits & SIGN_MASK | ((e + BIAS) as u128) << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    #[test]
    fn log1p_fast_leg_is_sound() {
        let unit: Float = Float::with_val(PRECISION, 2).pow(-214);
        let mut worst = 0.0;
        let mut worst_x = 0.0;

        for i in 0..SAMPLES {
            let x = sample_1p(i);
            let bits = x.to_bits();
            let magnitude = bits & !SIGN_MASK;

            if magnitude < SMALL || magnitude >= EXP_MASK || bits >= SIGN_MASK | ONE {
                continue;
            }
            let (e, j, d) = reduce1p(bits);
            let s = fast::<Natural>(e, j, z_fast(d));
            let truth = Float::with_val(PRECISION, x).ln_1p() / unit.clone();
            let ratio = Float::with_val(PRECISION, truth - value(s)).abs().to_f64()
                / Natural::ZIV_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        println!("log1pq general leg: worst |err|/gate = {worst:.4} at x={worst_x:?}");
        assert!(
            worst < 0.5,
            "log1pq gate covers only {:.2}× the slip at x={worst_x:?}",
            1.0 / worst
        );
    }

    /// [`small`]'s band: a random sign and significand, the exponent uniform
    /// over `[−113, −19]`.
    fn sample_small(i: u64) -> f128 {
        let bits = u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64;
        let e = ((bits >> 112 & 0x7fff) % 95) as i32 - 113;

        f128::from_bits(bits & SIGN_MASK | ((e + BIAS) as u128) << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    #[test]
    fn log1p_small_leg_is_sound() {
        let mut worst = 0.0;
        let mut worst_x = 0.0;

        for i in 0..SAMPLES {
            let x = sample_small(i);
            let bits = x.to_bits();
            let magnitude = bits & !SIGN_MASK;
            let m = magnitude & MANTISSA_MASK | IMPLICIT_BIT;
            let e = (magnitude >> EXP_SHIFT) as i32 - BIAS;
            let (high, low) = small_leg(m, e, bits >> 127 != 0);

            // The leg is `|log(1 + x)|·2^(254 − e)`; the gate is set on the
            // normalized field, so it shrinks by the product's leading zeros.
            let scale: Float = Float::with_val(PRECISION, 2).pow(254 - e);
            let truth = Float::with_val(PRECISION, x).ln_1p().abs() * scale;
            let got = Float::with_val(PRECISION, high) * Float::with_val(PRECISION, 2).pow(128)
                + Float::with_val(PRECISION, low);
            let gate = (SMALL_GATE as f64) * 2_f64.powi(15 - high.leading_zeros() as i32);
            let ratio = Float::with_val(PRECISION, truth - got).abs().to_f64() / gate;

            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        println!("log1pq small leg: worst |err|/gate = {worst:.4} at x={worst_x:?}");
        assert!(
            worst < 0.5,
            "log1pq small gate covers only {:.2}× the slip at x={worst_x:?}",
            1.0 / worst
        );
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
