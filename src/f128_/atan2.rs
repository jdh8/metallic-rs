//! The binary128 two-argument arc tangent.
//!
//! The reduction lives on *dyadic breakpoints*, so both sides of the divide are
//! exact integers:
//!
//! 1. **Sort.** `atan2` is odd in `y`, so the sign of `y` is peeled off and the
//!    magnitudes sorted: `t = min/max ∈ (0, 1]`, with a swap flag and the sign
//!    of `x` picking the quadrant offset 0, π/2, or π and whether `atan`
//!    enters the sum upward or downward.
//! 2. **Pick.** One float divide estimates `i = round(64·t)` to within 0.505,
//!    which is all a breakpoint needs — the reduction is exact for *any* `i`.
//! 3. **Reduce.** `tan(θ − atan(i/64)) = (64·N − i·D·2^dn) / (64·D·2^dn + i·N)`
//!    for significands `N/D` and exponent gap `dn`: because the breakpoint is
//!    a 6-bit dyadic, numerator and denominator are *exact* in 128-bit limbs
//!    (`dn ≤ 7` whenever `i ≥ 1`), and the reduced tangent `T` stays below
//!    2^-6.98.
//! 4. **Divide.** A float-seeded Newton reciprocal — no integer division
//!    anywhere — yields `T` to 128 bits (fast) or 384 bits (accurate), always
//!    from below, as a floating fraction with its own exponent.
//! 5. **Evaluate.** `atan(T) = T·(1 − Σ (−1)^k T^{2k}/(2k+3))`: the Taylor tail
//!    needs ten terms at the fast width, eighteen at the accurate one, in the
//!    same tiered fixed point as the rest of the family.
//! 6. **Add back.** `θ* = QOFF ± (atan(i/64) ± atan(T))` in a fixed-point frame
//!    — 256 bits at 2^-253 on the fast leg, 384 at 2^-381 on the accurate one,
//!    the former being the top limbs of the latter's tables.  `θ* ≥ 0.0078`
//!    there, so the frame is always normal.  When `i = 0` in the first quadrant
//!    there is no table term at all and `atan(t)` keeps its floating form the
//!    whole way, down to results that underflow to zero.
//! 7. **Round.** The fast leg rounds normal results on a fixed 15-bit guard and
//!    hands the tie window ([`ZIV_GATE`]) and every subnormal to the accurate
//!    leg, whose rounder handles the full grid including underflow.
//!
//! An exact ratio `|y/x| = i/64` collapses the whole result into table
//! constants; the generator audits that every such sum clears the nearest
//! rounding boundary by 2^-122 or more, so the accurate frame decides those
//! inputs outright.

use super::atan2_tables::{COEF, FRAC_3PI_4, PHI, QOFF};
use super::uint::{
    add_256, add_384, any_below, extract_u128, mhi, mhi_approx, mul_hi_64, mul_hi_256, mul_hi_384,
    shl_256, shl_384, shr_384_sat, sub_256, sub_384, wmul, wmul_128x256, wmul_128x384,
};
use super::{EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, QUIET_BIT, SIGN_MASK, split};
use core::f128::consts;

/// Discarded bits below the fast leg's 113-bit mantissa within its top limb.
const GUARD: u32 = 15;
const GUARD_MASK: u128 = (1 << GUARD) - 1;
const GUARD_HALF: u128 = 1 << (GUARD - 1);

/// Half-width of the rounding-tie window the fast leg refuses to decide, in
/// units of 2^-15 of the result's last bit.
///
/// The leg's slip is relative and its floor is the single-limb pipeline: the
/// two-unit reciprocal and the truncated quotient each cost a few times 2^-127
/// of `T`, the truncated `atan` products about as much again, the rounded `T²`
/// word and the tables far less.  Under 2^-123.5 relative in all, a sixth of
/// the gate at the worst ulp ratio — the margin certified in
/// [`ziv_soundness`].
const ZIV_GATE: u128 = 64;

/// The two-argument arc tangent, `atan(y/x)` in the quadrant of `(x, y)`.
#[must_use]
pub fn atan2q(y: f128, x: f128) -> f128 {
    let ybits = y.to_bits();
    let xbits = x.to_bits();
    let ay = ybits & !SIGN_MASK;
    let ax = xbits & !SIGN_MASK;

    // Zero, infinite, and NaN arguments all have quadrant-constant answers.
    if ay.wrapping_sub(1) >= EXP_MASK - 1 || ax.wrapping_sub(1) >= EXP_MASK - 1 {
        return edge(ybits, xbits);
    }
    let sign = ybits & SIGN_MASK;
    let r = reduce(ay, ax, xbits >> 127 != 0);
    let (frac, e2) = fast(&r);

    round_fast(frac, e2, sign).unwrap_or_else(|| accurate(&r, sign))
}

/// The arguments with nothing to approximate: zero, infinite, or NaN.
#[cold]
#[inline(never)]
fn edge(ybits: u128, xbits: u128) -> f128 {
    let ay = ybits & !SIGN_MASK;
    let ax = xbits & !SIGN_MASK;

    if ay > EXP_MASK {
        return f128::from_bits(ybits | QUIET_BIT);
    }
    if ax > EXP_MASK {
        return f128::from_bits(xbits | QUIET_BIT);
    }
    let sign = ybits & SIGN_MASK;
    let xneg = xbits >> 127 != 0;
    let pi = consts::PI.to_bits();

    if ay == EXP_MASK {
        let magnitude = if ax != EXP_MASK {
            consts::FRAC_PI_2.to_bits()
        } else if xneg {
            FRAC_3PI_4
        } else {
            consts::FRAC_PI_4.to_bits()
        };
        return f128::from_bits(sign | magnitude);
    }
    if ay == 0 {
        return f128::from_bits(sign | if xneg { pi } else { 0 });
    }
    if ax == 0 {
        return f128::from_bits(sign | consts::FRAC_PI_2.to_bits());
    }
    f128::from_bits(sign | if xneg { pi } else { 0 })
}

/// The exact integer reduction both legs share.
struct Reduction {
    /// `|64·N − i·D·2^dn|` in a nonzero sector, the small significand `N` when
    /// `i = 0` — normalized into `[2^127, 2^128)`, or zero on a breakpoint.
    numerator: u128,
    /// `64·D·2^dn + i·N` in a nonzero sector, the large significand `D` when
    /// `i = 0` — normalized into `[2^127, 2^128)`.
    denominator: u128,
    /// Binary exponent of the ratio net of both normalizations, so the reduced
    /// tangent is `(numerator/denominator)·2^scale`.
    scale: i32,
    /// The ratio sits below its breakpoint, so `atan(T)` enters downward.
    negative: bool,
    /// Breakpoint index: the reduction subtracts `atan(sector/64)`.
    sector: usize,
    /// Quadrant offset index into [`QOFF`]: 0, π/2, or π.
    quadrant: usize,
    /// The whole arc tangent enters the offset downward (`π/2 − θ`, `π − θ`).
    negate: bool,
}

impl Reduction {
    /// No table term at all: the result is `atan(t)` in floating form.
    const fn relative(&self) -> bool {
        self.sector == 0 && self.quadrant == 0
    }
}

fn reduce(ay: u128, ax: u128, xneg: bool) -> Reduction {
    let swap = ay > ax;
    let (big, small) = if swap { (ay, ax) } else { (ax, ay) };
    let (d, ex) = split(big);
    let (n, ey) = split(small);
    let dn = (ex - ey) as u32;

    // `i = round(64·t)` within 0.505 units, from 15-bit significand tops and
    // one pipelined float divide: |T| ≤ (0.505/64)/(1 + t·i/64) < 2^-6.98.
    // A gap of eight binades already lands `i = 0` with no divide at all.
    let i = if dn < 8 {
        let scale = f64::from_bits((1029 - u64::from(dn)) << 52);
        (scale * ((n >> 98) as u32 as f64) / ((d >> 98) as u32 as f64) + 0.5) as usize
    } else {
        0
    };
    debug_assert!(i <= 64);

    // A nonzero sector needs `64·t ≥ 0.5`, so `dn ≤ 7` and all three products
    // below are exact: `64·N < 2^119`, `i·D·2^dn < 2^126`, `64·D·2^dn + i·N < 2^127`.
    let (numerator, denominator, negative, scale) = if i == 0 {
        // Significands normalize by fifteen bits exactly, cancelling in `scale`.
        (n << 15, d << 15, false, -(dn as i32))
    } else {
        let far = ((d as i128) * (i as i128)) << dn;
        let kn = ((n as i128) << 6) - far;
        let magnitude = kn.unsigned_abs();
        let lzn = magnitude.leading_zeros();
        let kd = (d << (6 + dn)) + n * (i as u128);
        let lzd = kd.leading_zeros();
        (
            if magnitude == 0 { 0 } else { magnitude << lzn },
            kd << lzd,
            kn < 0,
            lzd as i32 - lzn as i32,
        )
    };
    Reduction {
        numerator,
        denominator,
        scale,
        negative,
        sector: i,
        quadrant: if swap { 1 } else { 2 * usize::from(xneg) },
        negate: swap != xneg,
    }
}

/// `(2^254/d)·(1 − δ)` with `0 ≤ δ < 2^-125`, for `d ∈ [2^127, 2^128)`: the
/// reciprocal both legs start from.  Every step in this family truncates
/// toward zero and Newton converges from below, so no iterate ever exceeds its
/// target — which keeps the refinements' residuals provably nonnegative.
///
/// The seed is a *hardware* 128-by-64 divide against the top limb, worth a
/// full 64 bits where an `f64` seed is worth 51 — so one Newton step lands
/// where two used to, and the third-order companion term (a dozen units wide
/// at this seed width, one 64-bit multiply) pays for the limb the divide
/// dropped.
fn recip_128(d: u128) -> u128 {
    let dh = d >> 64;
    let dl = d & u64::MAX as u128;
    // `r ∈ [2^191/d − 3, 2^191/d]`: dropping `dl` costs under two units and
    // the floor one more, so `−2` is what keeps the seed below its target.
    let r = ((1u128 << 127) / dh) - 2;
    // `e = ⌈(2^191 − d·r)/2^64⌉ < 3·2^64`, the residual in the seed's units.
    let e = (1u128 << 127) - (dh * r + ((dl * r) >> 64));
    // Newton's `r·e/2^64`, then Halley's `c·e/2^127` — the latter only needs
    // its top few bits, so both operands narrow to 32 bits first.
    let c = ((r * (e & u64::MAX as u128)) >> 64) + r * (e >> 64);
    let c2 = ((c >> 32) * (e >> 32)) >> 63;

    // The two floors can each round the correction up by one; `−1` is enough
    // to keep the whole iterate at or below `2^254/d`.
    (r << 63) + c + c2 - 1
}

/// `(2^382/d)·(1 − δ)` with `0 ≤ δ < 2^-249`: the accurate leg's base, one
/// full-width Newton doubling past [`recip_128`].
fn recip_256(d: u128) -> [u128; 2] {
    let r = recip_128(d);
    let (dh, dl) = wmul(d, r);
    let residual = sub_256([0, 1 << 126], [dl, dh]);
    let c = wmul_128x256(r, residual);

    add_256(
        [0, r],
        [(c[0] >> 126) | (c[1] << 2), (c[1] >> 126) | (c[2] << 2)],
    )
}

/// `(2^510/d)·(1 − δ)` with `0 ≤ δ < 2^-380`: one more Newton step.
fn recip_384(d: u128) -> [u128; 3] {
    let r = recip_256(d);
    let p = wmul_128x256(d, r);
    let residual = sub_384([0, 0, 1 << 126], p);
    let c = shl_256(mul_hi_256(r, [residual[0], residual[1]]), 2);

    add_384([0, r[0], r[1]], [c[0], c[1], 0])
}

/// The reduced ratio as a floating 128-bit fraction: the value is
/// `t1·2^(et−128)` with `t1 ∈ [2^127, 2^128)`, off by under 2^-125 of it —
/// slip the fast leg's gate absorbs with room certified in [`ziv_soundness`].
fn quotient_128(r: &Reduction) -> (u128, i32) {
    let (high, low) = wmul(r.numerator, recip_128(r.denominator));
    let lzp = high.leading_zeros();

    (top_256([low, high], lzp), r.scale + 2 - lzp as i32)
}

/// [`quotient_128`] at 384 bits, short by under 2^-370.
fn quotient_384(r: &Reduction) -> ([u128; 3], i32) {
    let p = wmul_128x384(r.numerator, recip_384(r.denominator));
    let lzp = p[3].leading_zeros();
    let frac = [
        (p[1] << lzp) | (p[0] >> (128 - lzp)),
        (p[2] << lzp) | (p[1] >> (128 - lzp)),
        (p[3] << lzp) | (p[2] >> (128 - lzp)),
    ];

    (frac, r.scale + 2 - lzp as i32)
}

/// `atan(T)` from the floating fraction: same scale out, at most one leading
/// zero in (`atan(T)/T > 1 − 2^-13.9`).
///
/// The Taylor sum runs latency-first: with `u = T²` rounded into a 2^-128 word
/// and `v = u²`, the even and odd halves of `Σ (−1)^k u^k/(2k+3)` are two
/// independent Horner chains in `v` — half the serial depth — and `T·u`
/// multiplies in while they run, so the chains' merge is the only work left.
/// Each half rides its `v²` terms in a 64-bit tail whose slack sits below
/// 2^-146 of the result.
fn atan_frac(t1: u128, et: i32) -> u128 {
    let sh = (-2 * et) as u32;
    if sh >= 128 {
        return t1;
    }
    let u = shr_round(mhi_approx(t1, t1), sh);
    let cube = mhi_approx(t1, u);
    let v = mhi_approx(u, u);
    let vh = (v >> 64) as u64;
    // The alternating series splits into two all-positive halves in `v`.
    let narrow = |k: usize| (COEF[k][2] >> 64) as u64;
    let tail_even = narrow(4) + mul_hi_64(vh, narrow(6) + mul_hi_64(vh, narrow(8)));
    let tail_odd = narrow(5) + mul_hi_64(vh, narrow(7) + mul_hi_64(vh, narrow(9)));
    let even = COEF[0][2] + mhi_approx(v, COEF[2][2] + mhi_approx(v, u128::from(tail_even) << 64));
    let odd = COEF[1][2] + mhi_approx(v, COEF[3][2] + mhi_approx(v, u128::from(tail_odd) << 64));

    t1 - mhi_approx(cube, even - mhi_approx(u, odd))
}

/// The fast leg: the result as a floating 128-bit fraction, i.e. the value is
/// `frac·2^(e2−128)` with `frac ∈ [2^127, 2^128)`.  The table sum runs in a
/// second limb below, which the rounder never reads — 15 guard bits sit inside
/// `frac` itself.
fn fast(r: &Reduction) -> (u128, i32) {
    if r.relative() {
        let (t1, et) = quotient_128(r);
        let f = atan_frac(t1, et);
        let lz = f.leading_zeros();
        return (f << lz, et - lz as i32);
    }
    let theta = if r.numerator == 0 {
        [0, 0]
    } else {
        let (t1, et) = quotient_128(r);
        let f = atan_frac(t1, et);
        // Into the 2^-253 frame: `f·2^(et−128)·2^253`, with `et ≥ −125` for a
        // nonzero sector and only the sectorless `i = 0` band reaching below.
        let position = et + 125;
        if position >= 0 {
            place_256(f, position as u32)
        } else if position > -128 {
            [f >> position.unsigned_abs(), 0]
        } else {
            [0, 0]
        }
    };
    // Both signs are coin flips on mixed quadrants: select by mask, not branch.
    let phi = [PHI[r.sector][1], PHI[r.sector][2]];
    let arc = add_signed_256(phi, theta, r.negative);
    let qoff = [QOFF[r.quadrant][1], QOFF[r.quadrant][2]];
    let s = add_signed_256(qoff, arc, r.negate);
    // θ* ∈ [0.0078, π] keeps the frame normal: at most nine leading zeros.
    let lz = s[1].leading_zeros();

    (top_256(s, lz), 3 - lz as i32)
}

/// The top 64 bits of `(high:low) << shift`, for `shift < 64`.  LLVM has no
/// 128-bit funnel shift, so every `u128` shift pair below would cost a `shld`,
/// a plain shift and a `cmov`; cut from 64-bit limbs each is one `shld`.
#[inline]
const fn funnel(low: u64, high: u64, shift: u32) -> u64 {
    (high << shift) | (low >> 1 >> (63 - shift))
}

/// The low 64 bits of `(high:low) >> shift`, for `shift < 64` — [`funnel`]'s
/// mirror, one `shrd`.
#[inline]
const fn funnel_down(low: u64, high: u64, shift: u32) -> u64 {
    (low >> shift) | (high << 1 << (63 - shift))
}

/// `x >> shift` rounded on the bit below it, for `0 < shift < 128`.  Adding
/// 2^(shift−1) first would be shorter but can carry out of the limb when `T`
/// is a breakpoint and the fraction is all ones.
#[inline]
const fn shr_round(x: u128, shift: u32) -> u128 {
    debug_assert!(shift > 0 && shift < 128);
    let bits = shift & 63;
    let [low, high] = if shift < 64 {
        [x as u64, (x >> 64) as u64]
    } else {
        [(x >> 64) as u64, 0]
    };
    let below = shift - 1;
    let word = if below < 64 {
        x as u64
    } else {
        (x >> 64) as u64
    };
    let bit = (word >> (below & 63)) & 1;

    (funnel_down(low, high, bits) as u128) | ((high >> bits) as u128) << 64 | bit as u128
}

/// `f << shift` in the 2^-253 frame, for `shift < 128`: the window starts in
/// limb 0 or 1, picked without shifting, and four funnels cut it out.
#[inline]
const fn place_256(f: u128, shift: u32) -> [u128; 2] {
    debug_assert!(shift < 128);
    let bits = shift & 63;
    let [a, b, c] = if shift < 64 {
        [f as u64, (f >> 64) as u64, 0]
    } else {
        [0, f as u64, (f >> 64) as u64]
    };

    [
        ((a << bits) as u128) | ((funnel(a, b, bits) as u128) << 64),
        (funnel(b, c, bits) as u128) | ((funnel(c, 0, bits) as u128) << 64),
    ]
}

/// The top 128 bits of `s << shift`, for `shift < 64`: all a normalization
/// ever keeps, so the low limb never needs shifting at all.
#[inline]
const fn top_256(s: [u128; 2], shift: u32) -> u128 {
    let low = (s[0] >> 64) as u64;
    let middle = s[1] as u64;
    let high = (s[1] >> 64) as u64;

    (funnel(low, middle, shift) as u128) | ((funnel(middle, high, shift) as u128) << 64)
}

/// `a + b` or `a − b` without a data-dependent branch: the subtrahend enters
/// in two's complement through an xor mask and a carry-in.
fn add_signed_256(a: [u128; 2], b: [u128; 2], negative: bool) -> [u128; 2] {
    let mask = 0u128.wrapping_sub(u128::from(negative));

    add_256(
        a,
        add_256([b[0] ^ mask, b[1] ^ mask], [u128::from(negative), 0]),
    )
}

/// Round the fast frame on its fixed 15-bit guard; `None` hands ties and the
/// subnormal range to the accurate leg.
fn round_fast(frac: u128, e2: i32, sign: u128) -> Option<f128> {
    if e2 < f128::MIN_EXP {
        return None;
    }
    let rest = frac & GUARD_MASK;

    if rest.abs_diff(GUARD_HALF) <= ZIV_GATE {
        return None;
    }
    // The gate has already ruled out a tie: the guard's top bit decides, and
    // the frame's discarded limb cannot flip it.
    Some(f128::from_bits(
        sign | ((((e2 + 16382) as u128) << EXP_SHIFT)
            + ((frac >> GUARD) - IMPLICIT_BIT)
            + u128::from(rest > GUARD_HALF)),
    ))
}

/// [`correction`] at 384 bits: eighteen Taylor terms in three tiers, each
/// riding enough powers of `u ≤ 2^-13.9` to bury its width's slack.
fn correction_384(t: &[u128; 3], et: i32) -> [u128; 3] {
    let sh = (-2 * et) as u32;
    if sh >= 384 {
        return [0; 3];
    }
    let u = shr_384_sat(mul_hi_384(*t, *t), sh);
    let mut narrow = COEF[17][2];

    for c in COEF[11..17].iter().rev() {
        narrow = c[2] - mhi(u[2], narrow);
    }
    let mut middle = [0, narrow];

    for c in COEF[2..11].iter().rev() {
        middle = sub_256([c[1], c[2]], mul_hi_256([u[1], u[2]], middle));
    }
    let mut q = [0, middle[0], middle[1]];

    for c in COEF[..2].iter().rev() {
        q = sub_384(*c, mul_hi_384(u, q));
    }
    mul_hi_384(u, q)
}

/// [`atan_frac`] at 384 bits.
fn atan_frac_384(t: [u128; 3], et: i32) -> [u128; 3] {
    sub_384(t, mul_hi_384(t, correction_384(&t, et)))
}

/// The 384-bit leg: everything the fast leg would not decide, including the
/// whole subnormal range.  The polynomial tail leaves about 2^-267 relative,
/// far past binary128 — hard ties beyond that depth would need a wider frame,
/// the same bound the rest of the binary128 family accepts.
#[cold]
#[inline(never)]
fn accurate(r: &Reduction, sign: u128) -> f128 {
    if r.relative() {
        let (t, et) = quotient_384(r);
        let f = atan_frac_384(t, et);
        let lz = f[2].leading_zeros();
        return round_384(shl_384(f, lz), et - lz as i32, sign);
    }
    let theta = if r.numerator == 0 {
        [0; 3]
    } else {
        let (t, et) = quotient_384(r);
        shr_384_sat(atan_frac_384(t, et), (3 - et) as u32)
    };
    let arc = if r.negative {
        sub_384(PHI[r.sector], theta)
    } else {
        add_384(PHI[r.sector], theta)
    };
    let s = if r.negate {
        sub_384(QOFF[r.quadrant], arc)
    } else {
        add_384(QOFF[r.quadrant], arc)
    };
    let lz = s[2].leading_zeros();

    round_384(shl_384(s, lz), 3 - lz as i32, sign)
}

/// Round a floating 384-bit fraction (`frac·2^(e2−384)`, `frac` normalized) to
/// binary128, ties to even, over the whole grid: normal, subnormal, and the
/// underflow to zero below half the least subnormal.
fn round_384(frac: [u128; 3], e2: i32, sign: u128) -> f128 {
    let keep = (e2 + 16494).min(113);

    if keep < 0 {
        return f128::from_bits(sign);
    }
    if keep == 0 {
        // In [2^-16495, 2^-16494): the least subnormal unless exactly halfway.
        return f128::from_bits(sign | u128::from(frac != [0, 0, 1 << 127]));
    }
    let (biased, implicit) = if keep == 113 {
        ((e2 + 16382) as u128, IMPLICIT_BIT)
    } else {
        (0, 0)
    };
    let shift = (384 - keep) as u32;
    let mantissa = extract_u128(frac, shift);
    let round_bit = frac[(shift as usize - 1) / 128] >> ((shift - 1) % 128) & 1;
    let up = round_bit != 0 && (any_below(frac, shift - 1) || mantissa & 1 != 0);

    f128::from_bits(sign | ((biased << EXP_SHIFT) + (mantissa - implicit) + u128::from(up)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrant_constants() {
        let pi = consts::PI;
        let half = consts::FRAC_PI_2;

        for y in [0.0_f128, -0.0] {
            let sign = if y.is_sign_negative() { -1.0 } else { 1.0_f128 };
            assert_eq!(atan2q(y, 1.0).to_bits(), (sign * 0.0).to_bits());
            assert_eq!(atan2q(y, 0.0).to_bits(), (sign * 0.0).to_bits());
            assert_eq!(atan2q(y, -1.0).to_bits(), (sign * pi).to_bits());
            assert_eq!(atan2q(y, -0.0).to_bits(), (sign * pi).to_bits());
            assert_eq!(atan2q(y, f128::INFINITY).to_bits(), (sign * 0.0).to_bits());
            assert_eq!(
                atan2q(y, f128::NEG_INFINITY).to_bits(),
                (sign * pi).to_bits()
            );
        }
        for y in [1.0_f128, f128::MIN_POSITIVE, f128::MAX] {
            assert_eq!(atan2q(y, 0.0).to_bits(), half.to_bits());
            assert_eq!(atan2q(-y, 0.0).to_bits(), (-half).to_bits());
            assert_eq!(atan2q(y, -0.0).to_bits(), half.to_bits());
            assert_eq!(atan2q(y, f128::INFINITY).to_bits(), 0.0_f128.to_bits());
            assert_eq!(atan2q(-y, f128::INFINITY).to_bits(), (-0.0_f128).to_bits());
            assert_eq!(atan2q(y, f128::NEG_INFINITY).to_bits(), pi.to_bits());
        }
        let inf = f128::INFINITY;
        assert_eq!(atan2q(inf, 1.0).to_bits(), half.to_bits());
        assert_eq!(atan2q(-inf, -1.0).to_bits(), (-half).to_bits());
        assert_eq!(atan2q(inf, inf).to_bits(), consts::FRAC_PI_4.to_bits());
        assert_eq!(
            atan2q(inf, -inf).to_bits(),
            f128::from_bits(FRAC_3PI_4).to_bits()
        );
        assert_eq!(
            atan2q(-inf, -inf).to_bits(),
            f128::from_bits(SIGN_MASK | FRAC_3PI_4).to_bits()
        );
        assert!(atan2q(f128::NAN, 1.0).is_nan());
        assert!(atan2q(1.0, f128::NAN).is_nan());
        assert!(atan2q(f128::NAN, f128::NAN).is_nan());
    }

    #[test]
    fn known_values() {
        assert_eq!(atan2q(1.0, 1.0).to_bits(), consts::FRAC_PI_4.to_bits());
        assert_eq!(atan2q(-1.0, 1.0).to_bits(), (-consts::FRAC_PI_4).to_bits());
        assert_eq!(atan2q(1.0, -1.0).to_bits(), FRAC_3PI_4);
        assert_eq!(
            atan2q(1.0, 2.0).to_bits(),
            0x3ffd_dac6_7056_1bb4_f68a_dfc8_8bd9_7875
        );
        assert_eq!(
            atan2q(2.0, 1.0).to_bits(),
            0x3fff_1b6e_192e_bbe4_46c6_d19a_a220_a39b
        );
        assert_eq!(
            atan2q(3.0, -7.0).to_bits(),
            0x4000_5e4c_36ca_0118_a2bf_4259_eee6_8f07
        );
        assert_eq!(
            atan2q(-3.0, -7.0).to_bits(),
            0xc000_5e4c_36ca_0118_a2bf_4259_eee6_8f07
        );
    }

    #[test]
    fn tiny_and_subnormal() {
        // Small enough that atan(t) rounds back to the exact quotient.
        let least = f128::from_bits(1);
        assert_eq!(atan2q(least, 1.0).to_bits(), least.to_bits());
        assert_eq!(atan2q(f128::MIN_POSITIVE, 2.0).to_bits(), 1 << 111);
        // A subnormal result with its own rounding: 2^-16400 / 3.
        assert_eq!(
            atan2q(f128::from_bits(1 << 94), 3.0).to_bits(),
            0x1555_5555_5555_5555_5555_5555
        );
        // atan(2^-16495) sits just below half the least subnormal.
        assert_eq!(atan2q(least, 2.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(atan2q(-least, 2.0).to_bits(), (-0.0_f128).to_bits());
        // Far past every grid: the true angle underflows to zero.
        assert_eq!(atan2q(least, f128::MAX).to_bits(), 0.0_f128.to_bits());
        // The other extreme aims at pi/2 from below.
        assert_eq!(
            atan2q(f128::MAX, least).to_bits(),
            consts::FRAC_PI_2.to_bits()
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

    /// A positive finite magnitude with the exponent field drawn from `range`.
    fn sample(i: u64, range: core::ops::RangeInclusive<u128>) -> f128 {
        let bits = mix128(i);
        let exponent = range.start() + (bits >> 112) % (range.end() - range.start() + 1);

        f128::from_bits(exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// Pairs covering banded and wide exponent gaps, sector interiors, and
    /// near-breakpoint cancellations; the caller adds the quadrants.
    fn draw(i: u64) -> (f128, f128) {
        let x = match i % 4 {
            0 => sample(2 * i, 16363..=16403),
            // Subnormal operands included: `split` normalizes them away.
            1 => sample(2 * i, 0..=32766),
            // A few ulps around an exact breakpoint ratio, where the reduced
            // tangent collapses and the table terms dominate.
            2 => {
                let y = sample(2 * i + 1, 16382..=16385);
                let near = (y / 64.0 * ((mix(i) % 64 + 1) as f128)).to_bits();
                let x = near.wrapping_add(mix(!i) as u128 % 15).wrapping_sub(7);
                return (y, f128::from_bits(x & !SIGN_MASK));
            }
            _ => sample(2 * i, 16380..=16386),
        };
        (sample(2 * i + 1, 16363..=16403), x)
    }

    /// `|frame − atan2(y, ±x)| / ZIV_GATE` in the gate's own units: gate units
    /// are 2^-15 of the fraction's last kept bit, i.e. 2^(e2−128) absolute.
    fn slip(y: f128, x: f128, xneg: bool) -> f64 {
        let r = reduce(y.to_bits(), x.to_bits(), xneg);
        let (frac, e2) = fast(&r);
        let frame = Float::with_val(PRECISION, frac) * Float::with_val(PRECISION, 2).pow(e2 - 128);
        let x = Float::with_val(PRECISION, if xneg { -x } else { x });
        let truth = Float::with_val(PRECISION, y).atan2(&x);
        let unit: Float = Float::with_val(PRECISION, 2).pow(e2 - 128);

        (Float::with_val(PRECISION, truth - frame).abs() / unit).to_f64() / ZIV_GATE as f64
    }

    /// Worst `|err|/gate` over the sample, both signs of `x`.
    #[test]
    fn fast_leg_is_sound() {
        let mut worst = 0.0;
        let mut worst_at = (0.0_f128, 0.0_f128, false);

        for i in 0..SAMPLES {
            let (y, x) = draw(i);

            for xneg in [false, true] {
                let ratio = slip(y, x, xneg);

                if ratio > worst {
                    worst = ratio;
                    worst_at = (y, x, xneg);
                }
            }
        }
        println!("atan2q fast leg: worst |err|/gate = {worst:.4} at {worst_at:?}");
        assert!(
            worst < 0.5,
            "atan2q gate covers only {:.2}× the slip at {worst_at:?}",
            1.0 / worst
        );
    }
}
