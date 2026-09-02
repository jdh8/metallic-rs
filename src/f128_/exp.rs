//! The binary128 exponential family, all reduced to one engine for 2<sup>y</sup>.
//!
//! `expq`, `exp2q` and `exp10q` differ only in the constant `L` of
//! `f(x) = 2^(x·L)`, so they share the whole pipeline:
//!
//! 1. **Reduce.** `y = x·L` is formed as an exact 384-bit integer product of the
//!    113-bit significand with a 256-bit `L`, then shifted into a fixed frame:
//!    an integer part `n` and a fraction `f ∈ [0, 1)`, 128 bits wide on the fast
//!    leg and 256 on the accurate one.  Two's complement handles `x < 0`, so
//!    `n = ⌊y⌋` and `f = y − n` come out of the same code path for both signs.
//! 2. **Look up.** The leading 18 bits of `f` index three 64-entry tables of
//!    2<sup>j/64</sup>, 2<sup>j/4096</sup> and 2<sup>j/262144</sup>, leaving
//!    `t < 2^-18` for the polynomial.
//! 3. **Evaluate.** `2^t − 1 = Σ ln(2)^k/k! · t^k` converges so fast at that
//!    width that the seventh term already sits below 2^-142.
//! 4. **Round.** The product of the three table entries with `2^t` is `2^f`,
//!    which is exactly `[1, 2)`, so no renormalization is needed except for the
//!    boundary carry.  `n` becomes the exponent field, and a normal result
//!    always discards the same 15 low bits — only overflow and the subnormal
//!    ladder need a variable shift, so they live in [`extreme`].
//!
//! Everything is unsigned fixed point: an `f128` multiply is soft-float on every
//! target that has no hardware binary128, so integer limbs are both faster and
//! exactly analyzable.  The fast leg carries 128 bits, 15 more than binary128
//! needs; when the discarded 15 bits sit within [`ZIV_GATE`] of a rounding tie
//! the accurate leg redoes the same steps at 256 bits.

use super::exp_tables::{COEF, INV_FACT, LOG2_10, LOG2E, ONE, Reduction, T0, T1, T2};
use super::uint::{
    add_256, add_384, leading_zeros_384, mhi, mul_hi_64, mul_hi_256, neg_384, shl_384, sub_256,
    wmul,
};
use super::{BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, QUIET_BIT, SIGN_MASK, split};

/// Half-width of the rounding-tie window the fast leg refuses to decide.
///
/// The fast leg's significand carries at most eight units of 2^-128 of error
/// (three table roundings, three truncated products, and the polynomial), so 32
/// units leave the 2× soundness margin certified in [`ziv_soundness`].
pub(super) const ZIV_GATE: u128 = 32;

/// The bits a normal result discards — the fast leg's 128 less binary128's 113
/// — with the mask and the tie of the rounding they decide.
pub(super) const GUARD: u32 = 15;
pub(super) const GUARD_MASK: u128 = (1 << GUARD) - 1;
pub(super) const GUARD_HALF: u128 = 1 << (GUARD - 1);

/// `|x| ≥ 2^15` overflows or underflows every member of the family.
const SATURATE: u128 = ((BIAS + 15) as u128) << EXP_SHIFT;

/// Below 2^-120 every member of the family rounds to 1: the largest slope in
/// play is `ln(10) < 2^1.2`, leaving `|f(x) − 1| < 2^-118.8`, well inside the
/// half ulp 2^-114 around 1.
const TINY: u128 = ((BIAS - 120) as u128) << EXP_SHIFT;

/// The exponential function, e<sup>x</sup>.
#[must_use]
pub fn expq(x: f128) -> f128 {
    exp_generic(x, &LOG2E)
}

/// The base-2 exponential function, 2<sup>x</sup>.
#[must_use]
pub fn exp2q(x: f128) -> f128 {
    exp_generic(x, &ONE)
}

/// The base-10 exponential function, 10<sup>x</sup>.
#[must_use]
pub fn exp10q(x: f128) -> f128 {
    exp_generic(x, &LOG2_10)
}

/// 2<sup>x·L</sup> for a reduction constant `L`.
#[inline]
fn exp_generic(x: f128, l: &Reduction) -> f128 {
    let bits = x.to_bits();
    let magnitude = bits & !SIGN_MASK;

    if magnitude >= SATURATE {
        if magnitude > EXP_MASK {
            return f128::from_bits(bits | QUIET_BIT);
        }
        return if bits & SIGN_MASK == 0 {
            f128::INFINITY
        } else {
            0.0
        };
    }
    if magnitude < TINY {
        return 1.0;
    }

    let (m, e) = split(magnitude);
    let negative = bits & SIGN_MASK != 0;
    let (n, f) = frame(m, e, l, negative);
    let (n, r) = fast(n, f);

    // A normal result always discards the same 15 bits, so the common path needs
    // no variable shift at all.  Overflow and the subnormal ladder do, but only
    // for 0.3% of the arguments, so their masks stay in [`extreme`].
    if (n - (f128::MIN_EXP - 1)) as u32 > (f128::MAX_EXP - f128::MIN_EXP) as u32 {
        return extreme(n, r, m, e, negative, l);
    }
    let rest = r & GUARD_MASK;

    if rest.abs_diff(GUARD_HALF) <= ZIV_GATE {
        return accurate(m, e, negative, l);
    }
    // The gate has already ruled out an exact tie, so the round bit decides.
    f128::from_bits(
        (((n + BIAS) as u128) << EXP_SHIFT)
            + ((r >> GUARD) - IMPLICIT_BIT)
            + u128::from(rest > GUARD_HALF),
    )
}

/// The results a normal one's 15-bit rounding does not cover: overflow, and the
/// subnormals that discard more.
#[cold]
#[inline(never)]
fn extreme(n: i32, r: u128, m: u128, e: i32, negative: bool, l: &Reduction) -> f128 {
    if undecided(n, r, ZIV_GATE) {
        return accurate(m, e, negative, l);
    }
    round(n, r, 0)
}

/// `|x|·L` as `[fraction low, fraction high, integer part]`, the fraction scaled
/// by 2^-256.
///
/// `head` is `L·2^252` truncated to 256 bits, so the exact product
/// `m · head = |x|·L · 2^(364 − e)` needs only a right shift to land in the
/// frame.  Truncating `L` biases `y` by at most `|y|·2^-252 < 2^-235`.
#[inline]
fn reduce(m: u128, e: i32, head: [u128; 2]) -> [u128; 3] {
    let (high, low) = wmul(m, head[0]);
    let (top, middle) = wmul(m, head[1]);
    let (middle, carry) = high.overflowing_add(middle);
    let product = [low, middle, top + u128::from(carry)];
    let shift = (108 - e) as u32;

    // `94 ≤ shift ≤ 228`, so the frame lands one or two limbs down: pick the
    // window, then funnel the rest.  `|y| < 1` once a whole limb is gone, which
    // is exactly the case whose integer limb is shifted in as zero.
    let [low, middle, high] = if shift < 128 {
        product
    } else {
        [product[1], product[2], 0]
    };
    let bits = shift & 127;

    // `<< 1 << (127 - bits)` is `<< (128 - bits)` with the no-op case in range.
    [
        (low >> bits) | (middle << 1 << (127 - bits)),
        (middle >> bits) | (high << 1 << (127 - bits)),
        high >> bits,
    ]
}

/// Bits `[shift, shift + 64)` of the 128-bit value `high:low`, for `shift < 64`.
///
/// `<< 1 << (63 - shift)` is `<< (64 - shift)` with the no-op case in range.
#[inline]
const fn funnel(low: u64, high: u64, shift: u32) -> u64 {
    (low >> shift) | (high << 1 << (63 - shift))
}

/// Trailing zeros of a reduction constant's 256-bit head.
#[inline]
const fn trailing_zeros(head: [u128; 2]) -> u32 {
    if head[0] == 0 {
        128 + head[1].trailing_zeros()
    } else {
        head[0].trailing_zeros()
    }
}

/// The fast leg's `(⌊y⌋, fraction scaled by 2^-128)` for `y = ±|x|·L`.
///
/// Only 143 bits of `m · head` survive the shift into the frame, so the window
/// is cut from 64-bit limbs: a variable 128-bit shift costs a pair of
/// conditional moves per limb, a 64-bit one is a single funnel.  The fraction
/// starts at bit `shift + 128` of the 384-bit product and the integer part at
/// `shift + 256`, which is limb `l + 2` and limb `l + 4` for `l = shift / 64`;
/// limb 6 and up are zero because the product is at most 369 bits wide.
///
/// Negating in two's complement is exactly the `⌊y⌋` / `y − ⌊y⌋` split for
/// negative `y`.  The borrow into the fraction is the *exact* test of whether
/// the product has any bit below the window: `m · head` is `head`'s trailing
/// zeros plus `m`'s, so it costs one count and one comparison — and folds away
/// entirely for `expq` and `exp10q`, whose heads end too close to odd for any
/// `m` to reach the window.  The complement itself is a mask rather than a branch: the
/// sign of a random argument is unpredictable, and a mispredict here costs more
/// than the whole negation.
#[inline]
fn frame(m: u128, e: i32, l: &Reduction, negative: bool) -> (i32, u128) {
    let (top, middle) = wmul(m, l.head[1]);
    let (middle, carry) = mhi(m, l.head[0]).overflowing_add(middle);
    let top = top + u128::from(carry);
    let shift = (108 - e) as u32;
    let bits = shift & 63;

    // `94 ≤ shift ≤ 228`, so the window starts at limb 3, 4 or 5.
    let [mut low, mut high, mut integer] = [(middle >> 64) as u64, top as u64, (top >> 64) as u64];

    if shift >= 128 {
        [low, high, integer] = [high, integer, 0];
    }
    if shift >= 192 {
        [low, high, integer] = [high, integer, 0];
    }
    let fraction =
        u128::from(funnel(low, high, bits)) | u128::from(funnel(high, integer, bits)) << 64;
    let mask = if negative { u128::MAX } else { 0 };
    let borrow = negative && m.trailing_zeros() + trailing_zeros(l.head) >= shift + 128;
    let (fraction, carry) = (fraction ^ mask).overflowing_add(u128::from(borrow));

    (
        ((integer >> bits) as i32 ^ mask as i32) + i32::from(carry),
        fraction,
    )
}

/// [`frame`]'s negation on a full 384-bit triple, for the accurate leg's whole
/// reduced fraction.
#[inline]
fn signed(y: [u128; 3], negative: bool) -> [u128; 3] {
    if negative { neg_384(y) } else { y }
}

/// The residual `L − head/2^252` of the reduction constant, in units of 2^-256
/// of the fraction.
fn correction(m: u128, e: i32, tail: u128) -> u128 {
    let shift = 236 - e;

    if shift >= 256 {
        0
    } else {
        mhi(m, tail) >> (shift - 128)
    }
}

/// 2<sup>f</sup> to 128 bits, as `(exponent bump, significand scaled by 2^127)`.
#[inline]
pub(super) fn fast(n: i32, f: u128) -> (i32, u128) {
    let (i0, i1, i2, t) = index(f);

    // The last three terms ride on `t^4 < 2^-72`, so 64-bit coefficients and a
    // 64-bit argument hold them to 2^-136 — a quarter of the multiplier work of
    // the 128-bit steps, which start where that no longer suffices.
    let short = (t >> 64) as u64;
    let tail = coefficient(3) + mul_hi_64(short, coefficient(4) + mul_hi_64(short, coefficient(5)));
    let mut q = COEF[2][1] + mhi(t, u128::from(tail) << 64);

    for c in COEF[..2].iter().rev() {
        q = c[1] + mhi(t, q);
    }
    // `2^t − 1 < 2^-18` at scale 2^-128 becomes `2^t` at scale 2^-127.
    let p = (1 << 127) + (mhi(t, q) >> 1);
    let r = mul127(mul127(T1[i1][1], T2[i2][1]), p);
    let (high, low) = wmul(T0[i0][1], r);
    let carry = high >> 127;

    // `2^f < 2` leaves the product one bit short of the frame, except when
    // rounding pushes it up to the boundary.
    (
        n + carry as i32,
        if carry == 0 {
            (high << 1) | (low >> 127)
        } else {
            high
        },
    )
}

/// [`fast`] at 256 bits, from the reduction up.
#[cold]
#[inline(never)]
fn accurate(m: u128, e: i32, negative: bool, l: &Reduction) -> f128 {
    let (n, r) = wide(m, e, negative, l);

    round(n, r[1], r[0])
}

/// 2<sup>x·L</sup> to 256 bits, as `(exponent bump, significand scaled by
/// 2^255)`.
fn wide(m: u128, e: i32, negative: bool, l: &Reduction) -> (i32, [u128; 2]) {
    let y = reduce(m, e, l.head);

    exp2_frame(signed(
        add_384(y, [correction(m, e, l.tail), 0, 0]),
        negative,
    ))
}

/// 2<sup>y</sup> to 256 bits from the reduced `y` itself: `[fraction low,
/// fraction high, integer part]`, the fraction scaled by 2^-256, as
/// `(exponent bump, significand scaled by 2^255)`.
pub(super) fn exp2_frame(y: [u128; 3]) -> (i32, [u128; 2]) {
    let (i0, i1, i2, t) = index(y[1]);
    let t = [y[0], t];
    let mut q = COEF[12];

    for c in COEF[..12].iter().rev() {
        q = add_256(*c, mul_hi_256(t, q));
    }
    let p = mul_hi_256(t, q);
    let p = [(p[0] >> 1) | (p[1] << 127), (p[1] >> 1) | 1 << 127];
    let r = mul255(mul255(T1[i1], T2[i2]), p);
    let r = mul_hi_256(T0[i0], r);
    let carry = r[1] >> 127;
    let r = if carry == 0 {
        [r[0] << 1, (r[1] << 1) | (r[0] >> 127)]
    } else {
        r
    };

    (y[2] as i32 + carry as i32, r)
}

/// The `k`-th Taylor coefficient truncated to 64 bits, scaled by 2^-64.
#[inline]
const fn coefficient(k: usize) -> u64 {
    (COEF[k][1] >> 64) as u64
}

/// Split the leading 18 bits of a fraction into the three table indices, and
/// return the `t < 2^-18` left for the polynomial.
#[inline]
const fn index(f: u128) -> (usize, usize, usize, u128) {
    (
        (f >> 122) as usize,
        (f >> 116) as usize & 63,
        (f >> 110) as usize & 63,
        f & ((1 << 110) - 1),
    )
}

/// Product of two significands scaled by 2^127, truncated back to that scale.
///
/// Sound only where the exact product stays below 2, which is why the table
/// levels multiply before the leading one.
#[inline]
fn mul127(a: u128, b: u128) -> u128 {
    let (high, low) = wmul(a, b);
    (high << 1) | (low >> 127)
}

/// [`mul127`] at 256 bits.
fn mul255(a: [u128; 2], b: [u128; 2]) -> [u128; 2] {
    let p = mul_hi_256(a, b);
    [p[0] << 1, (p[1] << 1) | (p[0] >> 127)]
}

/// Bits of the significand the final rounding discards, 128 or less.
#[inline]
fn discarded(n: i32) -> u32 {
    (15 + (f128::MIN_EXP - 1 - n).max(0)) as u32
}

/// Whether the fast leg's significand sits too close to a rounding tie to
/// decide the last bit.
///
/// `gate` is the half-width of the refused window, widened by callers whose
/// frame magnified the fast leg's error.
#[inline]
pub(super) fn undecided(n: i32, r: u128, gate: u128) -> bool {
    let shift = discarded(n);

    // Saturating results carry no significand bits to decide.
    if n > f128::MAX_EXP - 1 || shift > 128 {
        return false;
    }
    (r & (u128::MAX >> (128 - shift))).abs_diff(1 << (shift - 1)) <= gate
}

/// Round `(high + low·2^-128)·2^(n − 127)` to binary128, ties to even.
#[inline]
pub(super) fn round(n: i32, high: u128, low: u128) -> f128 {
    if n > f128::MAX_EXP - 1 {
        return f128::INFINITY;
    }
    let shift = discarded(n);

    // Below half the smallest subnormal every significand rounds to zero.
    if shift > 128 {
        return 0.0;
    }
    let rest = high & (u128::MAX >> (128 - shift));
    let half = 1 << (shift - 1);
    let mantissa = high.checked_shr(shift).unwrap_or(0);
    let up = rest > half || (rest == half && (low != 0 || mantissa & 1 != 0));
    let packed = if n >= f128::MIN_EXP - 1 {
        (((n + BIAS) as u128) << EXP_SHIFT) + (mantissa - IMPLICIT_BIT)
    } else {
        mantissa
    };

    // Adding the round bit to the packed form carries the significand into the
    // exponent, and `MAX` into `+∞`.
    f128::from_bits(packed + u128::from(up))
}

/// `x ≤ −80` rounds to −1: `e^-80 < 2^-115` sits inside the half ulp 2^-114
/// below 1.  The bound doubles as the guard that keeps `⌊x·log2(e)⌋ ≥ −128`,
/// so the frame of 1 never shifts the significand away entirely.
const MINUS_ONE: u128 = ((BIAS + 6) as u128) << EXP_SHIFT | 1 << (EXP_SHIFT - 2);

/// Below 2^-114 the quadratic term `x²/2 < 2^(2e+1)` stays under the half ulp
/// `2^(e−114)` of `x`, so `e^x − 1` rounds to `x` itself.
const TINY_EXPM1: u128 = ((BIAS - 114) as u128) << EXP_SHIFT;

/// `|y| < 2^-18` keeps [`expm1_small`]'s thirteen-term series converged; the
/// fallback routing tests it on the reduced fraction.
const POLY_LIMIT: u128 = 1 << 110;

/// `|x| ≥ 2^-6` goes to the engine's frame: `|e^x − 1| > 2^-6.02` there, which
/// [`subtract_one`] can normalize within [`MAX_SHIFT`].
const FRAME_EXP: i32 = -6;

/// `|x| ≥ 2^-19` needs [`expm1_mid`]'s fifteen Taylor terms; below,
/// [`expm1_tiny`]'s six reach the same depth.
const MID_EXP: i32 = -19;

/// Leading zeros of the subtracted frame the widened gate still covers: the
/// deepest cancellation [`FRAME_EXP`] admits is `1 − e^(−2^-6) > 2^-6.02`.
const MAX_SHIFT: u32 = 7;

const _: () = assert!(MINUS_ONE == 80.0_f128.to_bits());

/// e<sup>x</sup> − 1, correctly rounded where [`expq`] would cancel.
///
/// Near zero `e^x − 1 ≈ x`, so the engine's `2^f ∈ [1, 2)` is the wrong frame:
/// subtracting 1 from it is exact but leaves only `128 − |log2 x|` significant
/// bits.  The exponent of `x` picks one of three legs:
///
/// - `|x| ≥ 2^-6` — the engine's fast leg, and [`subtract_one`] normalizes the
///   difference, widening the Ziv gate by the shift it needed.
/// - `|x| < 2^-6` — no log2(e) reduction at all: `|e^x − 1| = a·(1 ± d)` for
///   `x = ±a` with `d = a·h(±a)` and `h(x) = (e^x − 1 − x)/x²`, so the
///   correction `d < 2^-7` rides *on top of* the exact significand of `x` and
///   no bit is ever subtracted away.  [`expm1_mid`] sums `h` as even and odd
///   halves of [`INV_FACT`]; below 2^-19 [`expm1_tiny`]'s six terms suffice.
/// - Behind either gate, a 256-bit leg decides on its own: [`expm1_small`]
///   for `|y| < 2^-18`, [`expm1_accurate`] elsewhere.
#[must_use]
pub fn expm1q(x: f128) -> f128 {
    let bits = x.to_bits();
    let magnitude = bits & !SIGN_MASK;
    let negative = bits & SIGN_MASK != 0;

    if magnitude >= SATURATE {
        if magnitude > EXP_MASK {
            return f128::from_bits(bits | QUIET_BIT);
        }
        return if negative { -1.0 } else { f128::INFINITY };
    }
    if negative && magnitude >= MINUS_ONE {
        return -1.0;
    }
    if magnitude < TINY_EXPM1 {
        return x;
    }

    let (m, e) = split(magnitude);

    if e >= FRAME_EXP {
        let (n, f) = frame(m, e, &LOG2E, negative);
        let decided = subtract_one(fast(n, f))
            .filter(|&(n, r, gate)| !undecided(n, r, gate))
            .map(|(n, r, _)| round(n, r, 0));
        let value = decided.unwrap_or_else(|| expm1_accurate(m, e, negative));
        return with_sign(value, negative);
    }
    let (n, r, low) = if e >= MID_EXP {
        expm1_mid(m << 15, e, negative)
    } else {
        expm1_tiny(m << 15, e, negative)
    };
    let value = if undecided(n, r, ZIV_GATE) {
        let y = reduce(m, e, LOG2E.head);

        if y[2] == 0 && y[1] < POLY_LIMIT {
            expm1_small(m, e, y, negative)
        } else {
            expm1_accurate(m, e, negative)
        }
    } else {
        round(n, r, low)
    };

    with_sign(value, negative)
}

/// `1/(j + 2)!` truncated to 64 bits, scaled by 2^-64.
#[inline]
const fn inv_factorial_64(j: usize) -> u64 {
    (INV_FACT[j] >> 64) as u64
}

/// `|e^x − 1|` for `2^-19 ≤ |x| < 2^-6`, as `(exponent, significand scaled by
/// 2^127, the bits below it)`.
///
/// `h(x) = (e^x − 1 − x)/x²` splits into even and odd sums of [`INV_FACT`] in
/// `w = a²` — two short all-positive Horner chains that overlap, with the sign
/// folded into the single combining add.
#[inline]
fn expm1_mid(m15: u128, e: i32, negative: bool) -> (i32, u128, u128) {
    let t = m15 >> (-1 - e) as u32;
    let w = mhi(t, t);
    let short = (w >> 64) as u64;

    // The last two terms of each chain ride on `w^6 < 2^-72`, so 64-bit limbs
    // hold them to 2^-136.
    let tail = inv_factorial_64(12) + mul_hi_64(short, inv_factorial_64(14));
    let mut even = INV_FACT[10] + mhi(w, u128::from(tail) << 64);

    for j in [8, 6, 4, 2, 0] {
        even = INV_FACT[j] + mhi(w, even);
    }
    let tail = inv_factorial_64(11) + mul_hi_64(short, inv_factorial_64(13));
    let mut odd = INV_FACT[9] + mhi(w, u128::from(tail) << 64);

    for j in [7, 5, 3, 1] {
        odd = INV_FACT[j] + mhi(w, odd);
    }
    // `h ≈ 1/2` dwarfs the odd half `a·Q(a²) < 2^-8`, so the difference stays
    // far from zero and the sign is a mask, not a branch.
    let mask = if negative { u128::MAX } else { 0 };
    let h = even.wrapping_add((mhi(t, odd) ^ mask).wrapping_sub(mask));

    apply_correction(m15, mhi(t, h), e, negative)
}

/// [`expm1_mid`] below 2^-19, where six terms of `h` already reach 2^-134: the
/// alternating series keeps every Horner step positive, so the sign folds into
/// masks instead of a signed frame.
#[inline]
fn expm1_tiny(m15: u128, e: i32, negative: bool) -> (i32, u128, u128) {
    let t = m15 >> (-1 - e) as u32;
    let mask = if negative { u128::MAX } else { 0 };
    let narrow = mask as u64;
    let short = (t >> 64) as u64;

    // Terms 4..=5 ride on `a^4 < 2^-76`, so 64-bit limbs hold them to 2^-140.
    let term = mul_hi_64(short, inv_factorial_64(5)) ^ narrow;
    let tail = inv_factorial_64(4).wrapping_add(term.wrapping_sub(narrow));
    let term = mhi(t, u128::from(tail) << 64) ^ mask;
    let mut h = INV_FACT[3].wrapping_add(term.wrapping_sub(mask));

    for j in [2, 1, 0] {
        let term = mhi(t, h) ^ mask;
        h = INV_FACT[j].wrapping_add(term.wrapping_sub(mask));
    }
    apply_correction(m15, mhi(t, h), e, negative)
}

/// `s·(1 ± d)` on the exact significand `s = m15/2^128` of `|x|`: `|e^x − 1|`
/// as `(exponent, significand scaled by 2^127, the bits below it)`, from the
/// correction `d = a·h(±a) < 2^-7` scaled by 2^128.
///
/// The product `m15·d` is exact at 256 bits and the sum moves the leading bit
/// at most one place either way, only when the significand sits within `d` of
/// a binade edge — both carry branches are rare and well predicted.  The sign
/// of `x` is a coin flip, so it stays mask arithmetic throughout.
#[inline]
fn apply_correction(m15: u128, d: u128, e: i32, negative: bool) -> (i32, u128, u128) {
    let (hi, lo) = wmul(m15, d);
    let mask = if negative { u128::MAX } else { 0 };
    let low = (lo ^ mask).wrapping_sub(mask);
    let signed = (hi ^ mask).wrapping_add(u128::from(negative && lo == 0));
    let (high, carried) = m15.overflowing_add(signed);

    // Two's complement wraps whenever the correction subtracts, so the true
    // carry out is the flag *relative to the sign*.
    if carried != negative {
        (e + 1, 1 << 127 | high >> 1, high << 127 | low >> 1)
    } else if high >> 127 != 0 {
        (e, high, low)
    } else {
        (e - 1, high << 1 | low >> 127, low << 1)
    }
}

/// `|2^n·r/2^127 − 1|` from the fast leg, as `(exponent, significand, gate)`.
///
/// Above 1 the tighter frame is that of `2^n`, below it that of the subtracted
/// one.  The leading zeros of the difference say how far the fast leg's error
/// was magnified, and the gate widens by exactly that shift — past
/// [`MAX_SHIFT`] no gate is left to widen.
#[inline]
const fn subtract_one((n, r): (i32, u128)) -> Option<(i32, u128, u128)> {
    let (v, frame) = if n >= 0 {
        // `n ≥ 128` puts the 1 below the frame.  The deficit is then under one
        // unit — far inside the gate — so any tie it could cross falls back.
        (r - if n < 128 { 1 << (127 - n) } else { 0 }, n)
    } else {
        ((r >> (-1 - n) as u32).wrapping_neg(), -1)
    };
    let shift = v.leading_zeros();

    if shift > MAX_SHIFT {
        return None;
    }
    Some((frame - shift as i32, v << shift, ZIV_GATE << shift))
}

/// [`expm1q`] at 256 bits for `|y| < 2^-18`, keeping relative rather than
/// absolute accuracy.
///
/// `e^x − 1 = x·G(x)` with `G(x) = log2(e)·Q(y)`, and the significand `m` is
/// exact, so the 384-bit product carries every bit of `G` plus a true sticky
/// below it.  That is what settles `x = 2^-112`, where `x²/2` lands *on* a
/// rounding tie and only the cubic term breaks it — 227 bits below the result,
/// where the subtracted frame of [`expm1_accurate`] has nothing left.
#[cold]
#[inline(never)]
fn expm1_small(m: u128, e: i32, y: [u128; 3], negative: bool) -> f128 {
    let t = [y[0], y[1]];
    let mut q = COEF[12];

    // `y < 0` alternates the series; every step still keeps `|t·q|` below the
    // next coefficient, so the unsigned frame holds.
    for c in COEF[..12].iter().rev() {
        let term = mul_hi_256(t, q);
        q = if negative {
            sub_256(*c, term)
        } else {
            add_256(*c, term)
        };
    }
    let g = mul_hi_256(q, LOG2E.head);
    let (high, low) = wmul(m, g[0]);
    let (top, middle) = wmul(m, g[1]);
    let (middle, carry) = high.overflowing_add(middle);

    // `|x| = m·2^(e−112)` and `G = g·2^-252`, so the product lands at 2^(e−364).
    let product = [low, middle, top + u128::from(carry)];
    let shift = leading_zeros_384(product);
    let p = shl_384(product, shift);

    round(e + 19 - shift as i32, p[2], p[1] | u128::from(p[0] != 0))
}

/// [`expm1q`] at 256 bits, from the reduction up.
///
/// Away from zero the subtracted frame still leaves 238 bits or more, so this
/// leg decides its half of the range on its own and needs no gate.
#[cold]
#[inline(never)]
fn expm1_accurate(m: u128, e: i32, negative: bool) -> f128 {
    let (n, r) = wide(m, e, negative, &LOG2E);
    let (v, frame) = if n >= 0 {
        let one = match n {
            // Past 255 the 1 sits below the frame; one unit is the closest
            // representable deficit and stays inside this leg's own error.
            256.. => [1, 0],
            128.. => [1 << (255 - n), 0],
            _ => [0, 1 << (127 - n)],
        };
        let (low, borrow) = r[0].overflowing_sub(one[0]);
        let high = r[1].wrapping_sub(one[1]).wrapping_sub(u128::from(borrow));
        ([low, high], n)
    } else {
        let k = (-1 - n) as u32;
        let shifted = [(r[0] >> k) | (r[1] << 1 << (127 - k)), r[1] >> k];
        let (low, borrow) = 0_u128.overflowing_sub(shifted[0]);
        let high = shifted[1].wrapping_neg().wrapping_sub(u128::from(borrow));
        ([low, high], -1)
    };

    // `|2^y − 1| ≥ 2^-114` keeps the leading limb of the difference nonzero.
    let shift = v[1].leading_zeros();
    let high = (v[1] << shift) | (v[0] >> 1 >> (127 - shift));

    round(frame - shift as i32, high, v[0] << shift)
}

/// Attach a sign to the magnitude [`round`] produced.
#[inline]
fn with_sign(magnitude: f128, negative: bool) -> f128 {
    f128::from_bits(magnitude.to_bits() | if negative { SIGN_MASK } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_and_special() {
        assert_eq!(expq(0.0).to_bits(), 1.0_f128.to_bits());
        assert_eq!(expq(-0.0).to_bits(), 1.0_f128.to_bits());
        assert_eq!(exp2q(10.0).to_bits(), 1024.0_f128.to_bits());
        assert_eq!(exp2q(-16382.0).to_bits(), f128::MIN_POSITIVE.to_bits());
        assert_eq!(exp2q(-16494.0).to_bits(), 1);
        assert_eq!(exp10q(3.0).to_bits(), 1000.0_f128.to_bits());
        assert_eq!(expq(f128::NEG_INFINITY).to_bits(), 0.0_f128.to_bits());
        assert!(expq(f128::INFINITY).is_infinite());
        assert!(expq(f128::NAN).is_nan());
        assert!(expq(f128::MAX).is_infinite());
        assert_eq!(expq(-f128::MAX).to_bits(), 0.0_f128.to_bits());
    }

    #[test]
    fn known_values() {
        // e and 1/e, correctly rounded.
        let e = f128::from_bits(0x4000_5bf0_a8b1_4576_9535_5fb8_ac40_4e7a);
        assert_eq!(expq(1.0).to_bits(), e.to_bits());
        assert_eq!(
            expq(-1.0).to_bits(),
            f128::from_bits(0x3ffd_78b5_6362_cef3_7c6a_eb7b_1e0a_4154).to_bits()
        );
        assert_eq!(exp2q(0.5).to_bits(), core::f128::consts::SQRT_2.to_bits());
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

    const PRECISION: u32 = 300;
    const SAMPLES: u64 = 200_000;

    fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A random significand and sign at an exponent uniform over `[-120, top]`,
    /// the whole span the reduction sees.
    fn sample(i: u64, top: i32) -> f128 {
        let bits = u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64;
        let exponent = (BIAS - 120) as u128 + (bits >> 120) % (top + 121) as u128;

        f128::from_bits(bits & SIGN_MASK | exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// The fast leg's raw `(exponent, significand)`, as [`exp_generic`] sees it.
    fn leg(x: f128, l: &Reduction) -> (i32, u128) {
        let (m, e) = split(x.to_bits() & !SIGN_MASK);
        let (n, f) = frame(m, e, l, x.is_sign_negative());

        fast(n, f)
    }

    /// Worst `|leg − f| / ZIV_GATE` over the sample, the error measured in units
    /// of the significand's last bit — exactly what the gate compares.
    fn worst_ratio(top: i32, l: &Reduction, f: impl Fn(&Float) -> Float) -> (f64, f128) {
        let mut worst = 0.0;
        let mut worst_x = 0.0;

        for i in 0..SAMPLES {
            let x = sample(i, top);
            let (n, r) = leg(x, l);
            let unit = Float::with_val(PRECISION, 2).pow(n - 127);
            let truth = f(&Float::with_val(PRECISION, x)) / unit;
            let ratio = Float::with_val(PRECISION, truth - Float::with_val(PRECISION, r))
                .abs()
                .to_f64()
                / ZIV_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        (worst, worst_x)
    }

    fn certify(name: &str, top: i32, l: &Reduction, f: impl Fn(&Float) -> Float) {
        let (worst, x) = worst_ratio(top, l, f);
        println!("{name} fast leg: worst |err|/gate = {worst:.4} at x={x:?}");
        assert!(
            worst < 0.5,
            "{name} gate covers only {:.2}× the slip at x={x:?}",
            1.0 / worst
        );
    }

    /// A random significand and sign at an exponent uniform over `[-114, 13]`,
    /// the whole span [`expm1q`]'s fast legs see.
    fn sample_expm1(i: u64) -> f128 {
        let bits = u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64;
        let exponent = (BIAS - 114) as u128 + (bits >> 120) % 128;

        f128::from_bits(bits & SIGN_MASK | exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// The raw `(exponent, significand, gate)` of whichever [`expm1q`] fast leg
    /// applies.
    fn expm1_leg(x: f128) -> Option<(i32, u128, u128)> {
        let (m, e) = split(x.to_bits() & !SIGN_MASK);
        let negative = x.is_sign_negative();

        if e >= FRAME_EXP {
            let (n, f) = frame(m, e, &LOG2E, negative);
            return subtract_one(fast(n, f));
        }
        let (n, r, _) = if e >= MID_EXP {
            expm1_mid(m << 15, e, negative)
        } else {
            expm1_tiny(m << 15, e, negative)
        };
        Some((n, r, ZIV_GATE))
    }

    /// Both [`expm1q`] fast legs at once: the gate travels with the leg, so one
    /// worst ratio certifies the widened frame gate and the near-zero one alike.
    #[test]
    fn expm1q_fast_legs_are_sound() {
        let mut worst = 0.0;
        let mut worst_x = 0.0;

        for i in 0..SAMPLES {
            let x = sample_expm1(i);

            // `x ≤ −80` returns −1 outright, before any leg runs.
            if x.to_bits() >= MINUS_ONE | SIGN_MASK {
                continue;
            }
            let Some((n, r, gate)) = expm1_leg(x) else {
                continue;
            };
            let unit = Float::with_val(PRECISION, 2).pow(n - 127);
            let truth = Float::with_val(PRECISION, x).exp_m1().abs() / unit;
            let ratio = Float::with_val(PRECISION, truth - Float::with_val(PRECISION, r))
                .abs()
                .to_f64()
                / gate as f64;

            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        println!("expm1q fast legs: worst |err|/gate = {worst:.4} at x={worst_x:?}");
        assert!(
            worst < 0.5,
            "expm1q gate covers only {:.2}× the slip at x={worst_x:?}",
            1.0 / worst
        );
    }

    #[test]
    fn expq_fast_leg_is_sound() {
        certify("expq", 13, &LOG2E, |x| x.clone().exp());
    }

    #[test]
    fn exp2q_fast_leg_is_sound() {
        certify("exp2q", 14, &ONE, |x| x.clone().exp2());
    }

    #[test]
    fn exp10q_fast_leg_is_sound() {
        certify("exp10q", 12, &LOG2_10, |x| x.clone().exp10());
    }
}
