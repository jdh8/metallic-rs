//! The binary128 tangent.
//!
//! The tangent is odd, so the sign of `x` is peeled off and its magnitude
//! rides [`sinq`](super::trig::sinq)'s Payne–Hanek reduction: the same
//! `n = round(256·|x|/π) mod 512`, breakpoint `j = n & 127`, and residual
//! `θ = g·π/2` with `|θ| ≤ π/512`, normalized at its own exponent.  What
//! differs is everything after:
//!
//! 1. **Evaluate.** `tan θ = θ·(1 + u·P(u))` for `u = θ²`, with `P` the
//!    Taylor sum `Σ T_{k+1}·u^k` of the tangent's all-positive coefficients
//!    `T_k = 2^(2k+2)·(2^(2k+2) − 1)·|B_{2k+2}|/(2k+2)!`: eight terms at the
//!    fast width as two even/odd chains in `v = u²`, twenty-five at the
//!    accurate one.  The sum keeps `θ`'s floating form, one carry at most.
//! 2. **Recombine.** With `A = j·π/256 + θ`, the addition formula
//!    `tan A = (T_j + tan θ)/(1 − T_j·tan θ)` needs one table of
//!    `T_j = tan(j·π/256) < 2^7` and one product.  Numerator and
//!    denominator live in a frame with seven integer bits — 256 bits at
//!    2^-249 on the fast leg, 384 at 2^-377 on the accurate one — and
//!    neither cancels: `T_1 > tan(π/512)` and `T_127·tan(π/512) < ½`.  An
//!    odd quadrant wants `tan(A + π/2) = −cot A`, which swaps the two and
//!    flips the sign.  At `j = 0` the numerator is `tan θ` itself in
//!    floating form, and only the odd quadrant divides at all.
//! 3. **Divide.** [`atan2q`](super::atan2)'s hardware-seeded Newton
//!    reciprocal of the denominator's top limb, then one Newton step *on the
//!    quotient* — the exact residual against the full denominator, times
//!    that reciprocal — lands within 2^-127 of the ratio on the fast leg;
//!    twice over at 384 bits, within 2^-377 on the accurate one.  Every
//!    truncation keeps the iterates below the ratio, so no residual ever
//!    goes negative.
//! 4. **Round.** `atan2q`'s rounders and its
//!    [`ZIV_GATE`](super::atan2::ZIV_GATE).
//!
//! Below `|x| = 2^-8` the argument is its own reduced angle and the series
//! alone is the fast leg; below 2^-57 the cubic term sits under half an ulp
//! and `tan x = x`.  No binary128 sits within 2^-124 of a multiple of π/2,
//! so the tangent neither overflows nor underflows.

use super::atan2::{add_signed_256, place_256, recip_128, round_384, round_fast, top_256};
use super::trig::{DIRECT, TINY, edge, reduce, reduce_wide, squares};
use super::trig_tables::{TAN, TAN_COEF};
use super::uint::{
    add_256, add_384, leading_zeros_384, mhi_approx, mul_hi_384, shl_384, shr_256_sat, shr_384_sat,
    sub_256, sub_384, wmul, wmul_128x384,
};
use super::{EXP_MASK, SIGN_MASK, split};

/// The tangent.
#[must_use]
pub fn tanq(x: f128) -> f128 {
    let bits = x.to_bits();
    let ax = bits & !SIGN_MASK;

    if ax >= EXP_MASK {
        return edge(bits, ax);
    }
    if ax < TINY {
        return x;
    }
    let sign = bits & SIGN_MASK;
    let (m, e) = split(ax);

    fast(m, e)
        .and_then(|(frac, e2, flip)| round_fast(frac, e2, sign ^ flip))
        .unwrap_or_else(|| accurate(m, e, sign))
}

/// The sign bit a negative factor contributes.
#[inline]
const fn flip(negative: bool) -> u128 {
    if negative { SIGN_MASK } else { 0 }
}

/// `tan θ` in `θ`'s own floating form: `t1·(1 + u·P)` with the even and odd
/// halves of `Σ TAN_COEF[k]·u^k` as two chains in `v`.  Eight terms: the
/// ninth is below 2^-143 for `u < 2^-14.7`.  `tan θ > θ` can carry into the
/// next binade, which only the top of the direct band reaches.
#[inline]
fn tan_frac(t1: u128, et: i32, u: u128, v: u128) -> (u128, i32) {
    let c = |k: usize| TAN_COEF[k][2];
    let a = c(0) + mhi_approx(v, c(2) + mhi_approx(v, c(4) + mhi_approx(v, c(6))));
    let b = c(1) + mhi_approx(v, c(3) + mhi_approx(v, c(5) + mhi_approx(v, c(7))));
    let p = a + mhi_approx(u, b);
    let (s, carry) = t1.overflowing_add(mhi_approx(mhi_approx(t1, u), p));
    let shift = u32::from(carry);

    ((s >> shift) | (u128::from(carry) << 127), et + shift as i32)
}

/// `n/d` for `n, d ∈ [2^127, 2^128)` as a floating fraction `q·2^(eq−128)`
/// with `q ∈ [2^127, 2^128)`, short of the ratio by under 2^-127 of it.
///
/// [`recip_128`] and the floored first quotient both sit below their
/// targets, so the residual `n·2^127 − d·q0` is nonnegative and under
/// 2^131.2; times the same reciprocal it is the correction, whose own slack
/// — the residual's four dropped bits, the reciprocal's 2^-125 of a
/// correction under 2^3.3 — sits far below one unit.  Only the final cut to
/// 128 bits costs anything.
#[inline]
fn quotient(n: u128, d: u128) -> (u128, i32) {
    let r = recip_128(d);
    let (high, low) = wmul(n, r);
    let q0 = (high << 1) | (low >> 127);
    let (ph, pl) = wmul(d, q0);
    let rem = sub_256([n << 127, n >> 1], [pl, ph]);
    debug_assert!(rem[1] < 1 << 4);
    let (ch, cl) = wmul((rem[0] >> 4) | (rem[1] << 124), r);
    let q = add_256([0, q0], [(cl >> 122) | (ch << 6), ch >> 122]);
    let lz = q[1].leading_zeros();

    (top_256(q, lz), 1 - lz as i32)
}

/// The fast leg: `|tan x|` as a floating 128-bit fraction `frac·2^(e2−128)`
/// with the sign it contributes, or `None` when the reduction cannot vouch
/// for it.  Shared with [`ziv_soundness`].
#[inline]
fn fast(m: u128, e: i32) -> Option<(u128, i32, u128)> {
    if e < DIRECT {
        let t1 = m << 15;
        let et = e + 1;
        let (u, v, _) = squares(t1, et);
        let (tt, et2) = tan_frac(t1, et, u, v);
        return Some((tt, et2, 0));
    }
    let r = reduce(m, e)?;
    let (u, v, _) = squares(r.t1, r.et);
    let (tt, et2) = tan_frac(r.t1, r.et, u, v);
    let odd = r.n & 128 != 0;
    let j = r.n & 127;

    if j == 0 {
        // The relative band: `tan θ` itself, or `−cot θ` a quadrant on.
        if !odd {
            return Some((tt, et2, flip(r.negative)));
        }
        let (f, eq) = quotient(1 << 127, tt);
        return Some((f, eq + 1 - et2, flip(!r.negative)));
    }
    // `T_j ± tan θ` and `1 ∓ T_j·tan θ` in the 2^-249 frame, exact but for
    // the product's bits below it; the fast leg keeps `θ ≥ 2^-57`, so every
    // shift is in range.
    let tj = [TAN[j][1], TAN[j][2]];
    let numerator = add_signed_256(tj, place_256(tt, (et2 + 121) as u32), r.negative);
    let (ph, pl) = wmul(TAN[j][2], tt);
    let product = shr_256_sat([pl, ph], (-et2) as u32);
    let denominator = add_signed_256([0, 1 << 121], product, !r.negative);
    // An odd quadrant wants `−cot A`: swap the two by mask, not by branch.
    let mask = 0u128.wrapping_sub(u128::from(odd));
    let x0 = (numerator[0] ^ denominator[0]) & mask;
    let x1 = (numerator[1] ^ denominator[1]) & mask;
    let n = [numerator[0] ^ x0, numerator[1] ^ x1];
    let d = [denominator[0] ^ x0, denominator[1] ^ x1];
    // Both stay above 2^-7.4 of the frame: at most fourteen leading zeros.
    let lzn = n[1].leading_zeros();
    let lzd = d[1].leading_zeros();
    let (f, eq) = quotient(top_256(n, lzn), top_256(d, lzd));

    Some((f, eq + lzd as i32 - lzn as i32, flip(odd)))
}

/// `tan θ` at 384 bits as a normalized floating fraction: twenty-five Taylor
/// terms by Horner (every step positive), on `θ/2` so the sum cannot carry
/// out of the frame — the halving drops one bit at 2^-384, far below what
/// the residual resolves.
fn tan_384(t: [u128; 3], et: i32, u: [u128; 3]) -> ([u128; 3], i32) {
    let mut q = TAN_COEF[TAN_COEF.len() - 1];

    for c in TAN_COEF[..TAN_COEF.len() - 1].iter().rev() {
        q = add_384(*c, mul_hi_384(u, q));
    }
    let half = shr_384_sat(t, 1);
    let s = add_384(half, mul_hi_384(half, mul_hi_384(u, q)));
    let lz = leading_zeros_384(s);

    (shl_384(s, lz), et + 1 - lz as i32)
}

/// `v + (n·2^382 − d·v)/d`: one Newton step on a 384-bit quotient `v` of
/// `n/d` from the reciprocal `r ≈ 2^510/d` of the denominator's top limb.
/// The residual's top 384 bits come from [`mul_hi_384`], short by up to four
/// units on top of the floor of `n/4`; a `margin` below that keeps the step
/// under the ratio whenever another step follows.
fn refine(n: [u128; 3], d: [u128; 3], v: [u128; 3], r: u128, margin: u128) -> [u128; 3] {
    let rem = sub_384(sub_384(shr_384_sat(n, 2), mul_hi_384(d, v)), [margin, 0, 0]);
    debug_assert!(rem[2] < 1 << 4);
    // `rem·2^384/d ≈ rem·r·2^-126`.
    let c = wmul_128x384(r, rem);

    add_384(
        v,
        [
            (c[0] >> 126) | (c[1] << 2),
            (c[1] >> 126) | (c[2] << 2),
            (c[2] >> 126) | (c[3] << 2),
        ],
    )
}

/// [`quotient`] at 384 bits: `n/d` for normalized `n, d` as a normalized
/// fraction `q·2^(eq−384)`, within 2^-377 of the ratio.  Two [`refine`]
/// steps from the top-limb reciprocal, held four units under [`recip_128`]
/// so it sits below `2^510/d` for the full-width `d` (the top limb alone
/// overstates the reciprocal by under 2^-127).
fn quotient_384(n: [u128; 3], d: [u128; 3]) -> ([u128; 3], i32) {
    let r = recip_128(d[2]) - 4;
    let p = wmul_128x384(r, n);
    let v = refine(n, d, [p[1], p[2], p[3]], r, 8);
    let v = refine(n, d, v, r, 0);
    let lz = leading_zeros_384(v);

    (shl_384(v, lz), 2 - lz as i32)
}

/// The 384-bit leg: everything the fast leg would not decide.
#[cold]
#[inline(never)]
fn accurate(m: u128, e: i32, sign: u128) -> f128 {
    let (n, negative, t, et) = if e < DIRECT {
        (0, false, [0, 0, m << 15], e + 1)
    } else {
        reduce_wide(m, e)
    };
    let u = shr_384_sat(mul_hi_384(t, t), (-2 * et) as u32);
    let (tt, et2) = tan_384(t, et, u);
    let odd = n & 128 != 0;
    let j = n & 127;

    if j == 0 {
        if !odd {
            return round_384(tt, et2, sign ^ flip(negative));
        }
        let (f, eq) = quotient_384([0, 0, 1 << 127], tt);
        return round_384(f, eq + 1 - et2, sign ^ flip(!negative));
    }
    // The same frame at 2^-377.
    let t = shr_384_sat(tt, (7 - et2) as u32);
    let product = shr_384_sat(mul_hi_384(TAN[j], tt), (-et2) as u32);
    let one = [0, 0, 1 << 121];
    let (numerator, denominator) = if negative {
        (sub_384(TAN[j], t), add_384(one, product))
    } else {
        (add_384(TAN[j], t), sub_384(one, product))
    };
    let (n, d) = if odd {
        (denominator, numerator)
    } else {
        (numerator, denominator)
    };
    let lzn = leading_zeros_384(n);
    let lzd = leading_zeros_384(d);
    let (f, eq) = quotient_384(shl_384(n, lzn), shl_384(d, lzd));

    round_384(f, eq + lzd as i32 - lzn as i32, sign ^ flip(odd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f128::consts;

    #[test]
    fn specials() {
        assert_eq!(tanq(0.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(tanq(-0.0).to_bits(), (-0.0_f128).to_bits());
        assert!(tanq(f128::INFINITY).is_nan());
        assert!(tanq(f128::NEG_INFINITY).is_nan());
        assert!(tanq(f128::NAN).is_nan());
        // The tiny band: the tangent is the argument.
        for x in [
            f128::from_bits(1),
            f128::MIN_POSITIVE,
            crate::f128_::exp2i(-58),
            f128::from_bits(TINY - 1),
        ] {
            assert_eq!(tanq(x).to_bits(), x.to_bits());
            assert_eq!(tanq(-x).to_bits(), (-x).to_bits());
        }
    }

    #[test]
    fn symmetry() {
        for x in [0.5_f128, 1.0, 3.0, 100.0, 1e30, crate::f128_::exp2i(-20)] {
            assert_eq!(tanq(-x).to_bits(), (-tanq(x)).to_bits());
        }
    }

    const GOLDEN_1: u128 = 0x3fff8eb245cbee3a5b8acc7d41323141;
    const GOLDEN_PI_4: u128 = 0x3fff0000000000000000000000000000;
    const GOLDEN_PI_2: u128 = 0x40711c46bd57277993a2ee60193c957b;
    const GOLDEN_PI: u128 = 0xbf8dcd129024e088a67cc74020bbea64;
    const GOLDEN_2M10: u128 = 0x3ff5000005555577777854854dedc28f;
    const GOLDEN_2P100: u128 = 0xbfffc86fbfa8cecc31efc7a12115f421;
    const GOLDEN_MAX: u128 = 0xc0008db7162c7114540eca807fae7391;
    const GOLDEN_3: u128 = 0xbffc23ef71254b86f0ccb0b27dff8543;

    #[test]
    fn known_values() {
        // mpmath golden values at 113-bit round-to-nearest.
        let check = |x: f128, want: u128| assert_eq!(tanq(x).to_bits(), want, "tan {x:?}");
        check(1.0, GOLDEN_1);
        check(consts::FRAC_PI_4, GOLDEN_PI_4);
        check(consts::FRAC_PI_2, GOLDEN_PI_2);
        check(consts::PI, GOLDEN_PI);
        check(crate::f128_::exp2i(-10), GOLDEN_2M10);
        check(crate::f128_::exp2i(100), GOLDEN_2P100);
        check(f128::MAX, GOLDEN_MAX);
        check(3.0, GOLDEN_3);
    }
}

/// MPFR certification that `atan2q`'s [`ZIV_GATE`] covers the fast leg's
/// true error with the 2× margin the project requires.  Run with
/// `CC=clang cargo +nightly test --release --features "f128 mpfr"`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::super::atan2::ZIV_GATE;
    use super::super::{BIAS, EXP_SHIFT, MANTISSA_MASK};
    use super::*;
    use rug::{Float, float::Constant, ops::Pow};

    const PRECISION: u32 = 300;
    const SAMPLES: u64 = 100_000;

    fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn mix128(i: u64) -> u128 {
        u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64
    }

    /// A positive magnitude with the unbiased exponent drawn from `range`.
    fn sample(i: u64, range: core::ops::RangeInclusive<i32>) -> f128 {
        let bits = mix128(i);
        let span = (range.end() - range.start() + 1) as u128;
        let exponent = (*range.start() + BIAS) as u128 + (bits >> 112) % span;

        f128::from_bits(exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// `round(k·π/2) + 2^-t` or `round(k·π/256) + 2^-t` for `k < 2^30` and
    /// `t ∈ [7, 60]`: residuals of about `2^-t` from a multiple of π/2 (the
    /// relative bands, `tan θ` and `−cot θ`) or from a breakpoint (where the
    /// placed `tan θ` sits deepest in the frame), spanning the fast leg's
    /// whole admissible range down to its hand-over.
    fn near_multiple(i: u64) -> f128 {
        let k = mix(i) % (1 << 30) + 1;
        let t = 7 + (mix(!i) % 54) as i32;
        let divisor: u32 = if i & 16 == 0 { 2 } else { 256 };
        let x = Float::with_val(PRECISION, Constant::Pi) * k / divisor;
        let x = x.to_f128_round(rug::float::Round::Nearest);

        x + crate::f128_::exp2i(-i64::from(t))
    }

    /// A binade edge: a mantissa within 2^-100 of a power of two from either
    /// side, where the series' carry and the quotient's binade move.
    fn edge(i: u64) -> f128 {
        let x = sample(i, -57..=20).to_bits();
        let fraction = (x & MANTISSA_MASK) >> 100;

        f128::from_bits(if i & 8 == 0 {
            x & !MANTISSA_MASK | fraction
        } else {
            x | MANTISSA_MASK - fraction
        })
    }

    fn draw(i: u64) -> f128 {
        match i % 5 {
            0 => sample(i, -57..=20),
            1 => sample(i, -57..=16383),
            2 => edge(i),
            _ => near_multiple(i),
        }
    }

    /// `|frame − |tan x|| / ZIV_GATE` in gate units of 2^(e2−128), or `None`
    /// where the fast leg hands over; the sign is checked on the way.
    fn slip(x: f128) -> Option<f64> {
        let (m, e) = split(x.to_bits() & !SIGN_MASK);
        let (frac, e2, flip) = fast(m, e)?;
        let truth = Float::with_val(PRECISION, x).tan();
        assert_eq!(flip != 0, truth.is_sign_negative(), "tanq sign at {x:?}");
        let unit: Float = Float::with_val(PRECISION, 2).pow(e2 - 128);
        let frame = Float::with_val(PRECISION, frac) * &unit;

        Some(
            (Float::with_val(PRECISION, truth.abs() - frame).abs() / unit).to_f64()
                / ZIV_GATE as f64,
        )
    }

    #[test]
    fn fast_leg_is_sound() {
        let mut worst = 0.0;
        let mut worst_at = 0.0_f128;
        let mut handed_over = 0;

        for i in 0..SAMPLES {
            let x = draw(i);
            let Some(ratio) = slip(x) else {
                handed_over += 1;
                continue;
            };
            if ratio > worst {
                worst = ratio;
                worst_at = x;
            }
        }
        println!(
            "tanq fast leg: worst |err|/gate = {worst:.4} at {worst_at:?} ({handed_over} handed over)"
        );
        assert!(
            worst < 0.5,
            "tanq gate covers only {:.2}× the slip at {worst_at:?}",
            1.0 / worst
        );
    }
}
