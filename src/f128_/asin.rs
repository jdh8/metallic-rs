//! The binary128 arc sine and arc cosine.
//!
//! Both are the [`atan2q`](super::atan2) pipeline fed a *wide* square root:
//! `asin(x) = atan2(x, √(1−x²))` and `acos(x) = atan2(√(1−x²), x)`.  The
//! pieces that made `atan2q` exact no longer are — one side of the ratio is
//! irrational — so the reduction runs wider instead:
//!
//! 1. **Square.** `1 − x²` is computed *exactly* (up to a one-sided ⌈·⌉ at
//!    2^-384): `x²` is an exact 226-bit integer, and the subtraction from 1
//!    cancels without error in fixed point.  Since `|x| ≤ 1 − 2^-113`, the
//!    difference stays above 2^-112 and the square root above 2^-57 — the
//!    ratio's exponents stay narrow even where `x` is subnormal.
//! 2. **Root.** An f64-seeded reciprocal square root refined by two of
//!    [`hypotq`](super::hypot)'s doubling steps reads the top 128 bits of
//!    `1 − x²`; one Newton step against the top 256 bits reaches ~2^-233
//!    ([`wide_sqrt`]), and the accurate leg takes one more against all 384
//!    bits to ~2^-355 ([`sqrt_384`]).
//! 3. **Reduce.** The same dyadic-breakpoint reduction as `atan2q`, but in
//!    256-bit (fast) or 384-bit (accurate) limbs, since the square-root side
//!    is no longer a 113-bit significand.  The fast leg then truncates the
//!    normalized numerator and denominator to their top 128 bits — a relative
//!    slip under 2^-126 on the reduced tangent — and rides `atan2q`'s
//!    [`fast`] leg, guard, and certified Ziv gate unchanged.  [`ziv_soundness`]
//!    re-certifies the gate for the widened error budget.
//! 4. **Refine.** The accurate leg re-derives everything from the 384-bit
//!    root: a fresh sort and sector, a 384-bit reduction, a Newton reciprocal
//!    of the wide denominator ([`recip_wide`]), and `atan2q`'s own
//!    [`atan_frac_384`], tables, and rounder.
//!
//! Tiny arguments need no special path: below 2^-128 the ⌈·⌉ saturates
//! `1 − x²` to `1 − 2^-384`, and the ~2^-256 slip that leaves on the ratio
//! keeps the result strictly between `x` and its half-ulp fence, where
//! `asin(x) = x + x³/6 + …` rounds to `x` regardless.

use super::atan2::{
    Reduction, assemble_384, atan_frac_384, fast, recip_128, round_384, round_fast,
};
use super::hypot::rsqrt_step;
use super::uint::{
    add_256, add_384, cmp_384, leading_zeros_256, leading_zeros_384, mhi, mul_hi_384, neg_384,
    shl_256, shl_384, shr_256_sat, shr_384_sat, sub_256, sub_384, wmul,
};
use super::{EXP_MASK, EXP_SHIFT, QUIET_BIT, SIGN_MASK, split};
use core::f128::consts;

/// The bit pattern of `1.0`.
const ONE: u128 = 0x3fff << EXP_SHIFT;

/// The arc sine.
#[must_use]
pub fn asinq(x: f128) -> f128 {
    let bits = x.to_bits();
    let ax = bits & !SIGN_MASK;
    let sign = bits & SIGN_MASK;

    if ax == 0 {
        return x;
    }
    if ax >= ONE {
        return edge(
            bits,
            ax,
            f128::from_bits(sign | consts::FRAC_PI_2.to_bits()),
        );
    }
    let (m, e) = split(ax);
    arc(m, e, sign, false, false)
}

/// The arc cosine.
#[must_use]
pub fn acosq(x: f128) -> f128 {
    let bits = x.to_bits();
    let ax = bits & !SIGN_MASK;
    let xneg = bits >> 127 != 0;

    if ax == 0 {
        return consts::FRAC_PI_2;
    }
    if ax >= ONE {
        return edge(bits, ax, if xneg { consts::PI } else { 0.0 });
    }
    let (m, e) = split(ax);
    arc(m, e, 0, true, xneg)
}

/// `|x| ≥ 1`: the exact endpoints, NaN passthrough, and the out-of-domain NaN.
#[cold]
#[inline(never)]
fn edge(bits: u128, ax: u128, endpoint: f128) -> f128 {
    if ax == ONE {
        return endpoint;
    }
    if ax > EXP_MASK {
        return f128::from_bits(bits | QUIET_BIT);
    }
    f128::NAN
}

/// The shared pipeline for `0 < |x| < 1`: `x = m·2^(e−112)`, with `sign`
/// applied to the result (asin only) and `xneg` steering the acos quadrant.
fn arc(m: u128, e: i32, sign: u128, acos: bool, xneg: bool) -> f128 {
    let sq = wide_sqrt(m, e);
    let r = reduce(m << 15, e + 1, &sq, acos, xneg);
    let (frac, e2) = fast(&r);

    round_fast(frac, e2, sign).unwrap_or_else(|| accurate(m << 15, e + 1, &sq, acos, xneg, sign))
}

/// `√(1 − x²)` carried wide, plus the pieces the accurate leg reuses.
struct Sqrt {
    /// `(1 − x²)·2^(384+2·shift)`, normalized into `[2^382, 2^384)` by an even
    /// shift, with the square ⌈·⌉ed at 2^-384 so the value never overshoots.
    vn: [u128; 3],
    /// `≈ 2^190/√vn[2]`, the reciprocal square root both Newton steps divide
    /// by, good to ~2^-119.
    q: u128,
    /// `√(vn·2^-384)·2^256` unnormalized (clamped to all-ones at the top),
    /// within ~2^-233 relative after one Newton step.
    frame: [u128; 2],
    /// [`Sqrt::frame`] normalized into `[2^255, 2^256)`.
    s: [u128; 2],
    /// `√(1−x²) = (s·2^-256)·2^es`.
    es: i32,
    /// [`Sqrt::es`] before the ±1 normalization of `frame`.
    es_base: i32,
}

/// `⌈x²·2^384⌉` and one Newton step of `√· ` against its top 256 bits.
fn wide_sqrt(m: u128, e: i32) -> Sqrt {
    let (hi, lo) = wmul(m, m);
    // x² = m²·2^(2e−224), so x²·2^384 shifts m² by 2e+160 — exactly when left.
    let shift = 2 * e + 160;
    let xx = if shift >= 0 {
        debug_assert!(shift <= 158);
        shl_384([lo, hi, 0], shift as u32)
    } else {
        ceil_shr(hi, lo, shift.unsigned_abs())
    };
    // 1 − x² ≥ 2^-112·(1 − 2^-113) keeps at most 112 leading zeros.
    let v = neg_384(xx);
    let lz = leading_zeros_384(v);
    debug_assert!(lz <= 112);
    let parity = lz & !1;
    let vn = shl_384(v, parity);
    let w = vn[2];

    // The f64 seed and doubling steps are `hypotq`'s exact-tier recipe.
    let seed = crate::exp2i(158) / ((w >> 64) as u64 as f64).sqrt();
    let q = rsqrt_step(w, rsqrt_step(w, seed as u128));

    // s0h ≈ √(vn·2^-384)·2^128; the shift clamps when the root grazes 1.
    let s0 = mhi(w, q);
    let s0h = if s0 >> 126 == 0 { s0 << 2 } else { u128::MAX };

    // One Newton step: frame = s0h·2^128 + (vtop − s0h²)·q/2^127, since
    // q·2^-127 ≈ 1/(2√z) in the frame's own units.
    let (sqh, sql) = wmul(s0h, s0h);
    let (h, downward) = sub_signed_256(&[vn[1], vn[2]], &[sql, sqh]);
    debug_assert!(h[1] < 1 << 18);
    let hs = (h[1] << 110) | (h[0] >> 18);
    let (p1, p0) = wmul(hs, q);
    let corr = [(p1 << 19) | (p0 >> 109), p1 >> 109];
    let frame = if downward {
        sub_256([0, s0h], corr)
    } else {
        let sum = add_256([0, s0h], corr);
        if sum[1] < s0h { [u128::MAX; 2] } else { sum }
    };

    let lzs = leading_zeros_256(frame);
    debug_assert!(lzs <= 1);
    #[allow(clippy::cast_possible_wrap)]
    let es_base = -((parity / 2) as i32);

    Sqrt {
        vn,
        q,
        frame,
        s: shl_256(frame, lzs),
        es: es_base - lzs as i32,
        es_base,
    }
}

/// One more Newton step against the full 384-bit `1 − x²`, to ~2^-355:
/// the accurate leg's root, normalized, with its exponent.
fn sqrt_384(sq: &Sqrt) -> ([u128; 3], i32) {
    let f3 = [0, sq.frame[0], sq.frame[1]];
    let square = square_top(sq.frame);
    let downward = cmp_384(sq.vn, square).is_lt();
    let h = if downward {
        sub_384(square, sq.vn)
    } else {
        sub_384(sq.vn, square)
    };
    debug_assert!(h[1] < 1 << 28 && h[2] == 0);
    let hs = (h[1] << 100) | (h[0] >> 28);
    let (p1, p0) = wmul(hs, sq.q);
    let corr = [(p1 << 29) | (p0 >> 99), p1 >> 99, 0];
    let s3 = if downward {
        sub_384(f3, corr)
    } else {
        let sum = add_384(f3, corr);
        if sum[2] < f3[2] { [u128::MAX; 3] } else { sum }
    };
    let lz = leading_zeros_384(s3);
    debug_assert!(lz <= 1);

    (shl_384(s3, lz), sq.es_base - lz as i32)
}

/// The top 384 bits of `frame²`, i.e. `frame²·2^-128` with only the lowest
/// cross-limb half dropped (under one unit short).
fn square_top(frame: [u128; 2]) -> [u128; 3] {
    let (h11, l11) = wmul(frame[1], frame[1]);
    let (h01, l01) = wmul(frame[0], frame[1]);
    let cross = [l01 << 1, (h01 << 1) | (l01 >> 127), h01 >> 127];

    add_384(
        add_384([0, l11, h11], cross),
        [mhi(frame[0], frame[0]), 0, 0],
    )
}

/// `⌈(hi:lo)·2^-shift⌉` for `shift ≥ 1`, saturating the ceiling at 1.
fn ceil_shr(hi: u128, lo: u128, shift: u32) -> [u128; 3] {
    if shift >= 256 {
        return [1, 0, 0];
    }
    let floor = shr_256_sat([lo, hi], shift);
    let dropped = if shift >= 128 {
        lo != 0 || hi & ((1 << (shift - 128)) - 1) != 0
    } else {
        lo & ((1 << shift) - 1) != 0
    };
    let r = add_256(floor, [u128::from(dropped), 0]);

    [r[0], r[1], 0]
}

/// `|a − b|` at 256 bits with the borrow direction.
fn sub_signed_256(a: &[u128; 2], b: &[u128; 2]) -> ([u128; 2], bool) {
    let negative = b[1] > a[1] || (b[1] == a[1] && b[0] > a[0]);

    if negative {
        (sub_256(*b, *a), true)
    } else {
        (sub_256(*a, *b), false)
    }
}

/// `a·k` for a 256-bit `a` below 2^243 and `k ≤ 2^13`.
fn mul_small_256(a: [u128; 2], k: u128) -> [u128; 2] {
    debug_assert!(a[1] < 1 << 115 && k <= 1 << 13);
    let (high, low) = wmul(a[0], k);

    [low, a[1] * k + high]
}

/// [`mul_small_256`] at 384 bits, for `a` below 2^371.
fn mul_small_384(a: [u128; 3], k: u128) -> [u128; 3] {
    debug_assert!(a[2] < 1 << 115 && k <= 1 << 13);
    let (h0, l0) = wmul(a[0], k);
    let (h1, l1) = wmul(a[1], k);
    let (l1, carry) = l1.overflowing_add(h0);

    [l0, l1, a[2] * k + h1 + u128::from(carry)]
}

/// `atan2q`'s quadrant plumbing: `(quadrant, negate)` from the sort and the
/// sign of the `x` argument.
const fn plumbing(swap: bool, xneg: bool) -> (usize, bool) {
    (if swap { 1 } else { 2 * (xneg as usize) }, swap != xneg)
}

/// The 64·`i` sector estimate shared by both legs: `i = round(64·t)` within
/// 0.505 units from 15-bit operand tops, exactly as `atan2q`'s reduce.
fn sector(ntop: u128, dtop: u128, dn: u32) -> usize {
    if dn >= 8 {
        return 0;
    }
    let scale = f64::from_bits((1029 - u64::from(dn)) << 52);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let i = (scale * ((ntop >> 113) as u32 as f64) / ((dtop >> 113) as u32 as f64) + 0.5) as usize;
    debug_assert!(i <= 64);
    i
}

/// The fast leg's 256-bit reduction, truncated into `atan2q`'s [`Reduction`].
fn reduce(fx: u128, ex: i32, sq: &Sqrt, acos: bool, xneg: bool) -> Reduction {
    let x256 = [0, fx];
    let x_big = ex > sq.es || (ex == sq.es && fx > sq.s[1]);
    let (d, ed, n, en) = if x_big {
        (x256, ex, sq.s, sq.es)
    } else {
        (sq.s, sq.es, x256, ex)
    };
    #[allow(clippy::cast_sign_loss)]
    let dn = (ed - en) as u32;
    let i = sector(n[1], d[1], dn);
    let swap = x_big != acos;
    let (quadrant, negate) = plumbing(swap, xneg);

    let (numerator, denominator, negative, scale) = if i == 0 {
        (n[1], d[1], false, -(dn as i32))
    } else {
        // Shifting both sides down 14 bits buys the headroom `atan2q` had for
        // free from 113-bit significands: `kd ≤ D·2^(7+dn) < 2^256`.
        let n14 = shr_256_sat(n, 14);
        let d14 = shr_256_sat(d, 14);
        let far = mul_small_256(d14, (i as u128) << dn);
        let (kn, negative) = sub_signed_256(&shl_256(n14, 6), &far);
        let kd = add_256(shl_256(d14, 6 + dn), mul_small_256(n14, i as u128));
        let lzd = leading_zeros_256(kd);

        if kn == [0, 0] {
            (0, shl_256(kd, lzd)[1], negative, 0)
        } else {
            let lzn = leading_zeros_256(kn);
            (
                shl_256(kn, lzn)[1],
                shl_256(kd, lzd)[1],
                negative,
                lzd as i32 - lzn as i32,
            )
        }
    };
    Reduction {
        numerator,
        denominator,
        scale,
        negative,
        sector: i,
        quadrant,
        negate,
    }
}

/// `(2^767/d)·(1 − δ)` with `0 ≤ δ < 2^-378`, for `d ∈ [2^383, 2^384)`: a
/// [`recip_128`] seed of the top limb and two wide Newton steps.  Every
/// iterate stays below its target — the seed backs off four units for the
/// dropped low limbs, and each step's correction backs off 32 for
/// [`mul_hi_384`]'s deficit inflating the residual — so the residual
/// subtraction never wraps.
fn recip_wide(d: [u128; 3]) -> [u128; 3] {
    let mut r = [0, 0, (recip_128(d[2]) - 4) << 1];

    for _ in 0..2 {
        let residual = sub_384([0, 0, 1 << 127], mul_hi_384(d, r));
        let correction = shl_384(mul_hi_384(r, residual), 1);
        r = sub_384(add_384(r, correction), [32, 0, 0]);
    }
    r
}

/// The accurate leg: everything re-derived from the 384-bit root, ending in
/// `atan2q`'s own series, tables, and rounder.
#[cold]
#[inline(never)]
fn accurate(fx: u128, ex: i32, sq: &Sqrt, acos: bool, xneg: bool, sign: u128) -> f128 {
    let (s3, es3) = sqrt_384(sq);
    let x3 = [0, 0, fx];
    let x_big = ex > es3 || (ex == es3 && cmp_384(x3, s3).is_gt());
    let (d, ed, n, en) = if x_big {
        (x3, ex, s3, es3)
    } else {
        (s3, es3, x3, ex)
    };
    #[allow(clippy::cast_sign_loss)]
    let dn = (ed - en) as u32;
    let i = sector(n[2], d[2], dn);
    let swap = x_big != acos;
    let (quadrant, negate) = plumbing(swap, xneg);

    let (kn, kd, negative, scale) = if i == 0 {
        (n, d, false, -(dn as i32))
    } else {
        let n14 = shr_384_sat(n, 14);
        let d14 = shr_384_sat(d, 14);
        let far = mul_small_384(d14, (i as u128) << dn);
        let six = shl_384(n14, 6);
        let negative = cmp_384(far, six).is_gt();
        let kn = if negative {
            sub_384(far, six)
        } else {
            sub_384(six, far)
        };
        let kd = add_384(shl_384(d14, 6 + dn), mul_small_384(n14, i as u128));
        let lzd = leading_zeros_384(kd);

        if kn == [0; 3] {
            ([0; 3], shl_384(kd, lzd), negative, 0)
        } else {
            let lzn = leading_zeros_384(kn);
            (
                shl_384(kn, lzn),
                shl_384(kd, lzd),
                negative,
                lzd as i32 - lzn as i32,
            )
        }
    };
    if kn == [0; 3] {
        return assemble_384([0; 3], negative, i, quadrant, negate, sign);
    }
    // The truncated quotient of two normalized fractions lands in
    // (2^382·(1 − ε), 2^384): usually one leading zero, two exactly at ½.
    let t = mul_hi_384(kn, recip_wide(kd));
    let lz = t[2].leading_zeros();
    let tn = shl_384(t, lz);
    let et = scale + 1 - lz as i32;
    let f = atan_frac_384(tn, et);

    if i == 0 && quadrant == 0 {
        let lzf = f[2].leading_zeros();
        return round_384(shl_384(f, lzf), et - lzf as i32, sign);
    }
    #[allow(clippy::cast_sign_loss)]
    let theta = shr_384_sat(f, (3 - et) as u32);

    assemble_384(theta, negative, i, quadrant, negate, sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_and_specials() {
        let half = consts::FRAC_PI_2;
        assert_eq!(asinq(0.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(asinq(-0.0).to_bits(), (-0.0_f128).to_bits());
        assert_eq!(asinq(1.0).to_bits(), half.to_bits());
        assert_eq!(asinq(-1.0).to_bits(), (-half).to_bits());
        assert!(asinq(1.5).is_nan());
        assert!(asinq(-1.5).is_nan());
        assert!(asinq(f128::INFINITY).is_nan());
        assert!(asinq(f128::NAN).is_nan());

        assert_eq!(acosq(1.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(acosq(-1.0).to_bits(), consts::PI.to_bits());
        assert_eq!(acosq(0.0).to_bits(), half.to_bits());
        assert_eq!(acosq(-0.0).to_bits(), half.to_bits());
        assert!(acosq(1.5).is_nan());
        assert!(acosq(f128::NEG_INFINITY).is_nan());
        assert!(acosq(f128::NAN).is_nan());
    }

    #[test]
    fn known_values() {
        // MPFR/mpmath golden values at 113-bit round-to-nearest.
        assert_eq!(asinq(0.5).to_bits(), consts::FRAC_PI_6.to_bits());
        assert_eq!(asinq(-0.5).to_bits(), (-consts::FRAC_PI_6).to_bits());
        assert_eq!(
            asinq(0.75).to_bits(),
            0x3ffe_b235_315c_680d_c081_583d_b360_d5e2
        );
        assert_eq!(
            asinq(-0.375).to_bits(),
            0xbffd_899f_4edc_962d_304f_a465_c06d_f1b6
        );
        assert_eq!(
            asinq(0.0625 + crate::f128_::exp2i(-70)).to_bits(),
            0x3ffb_002a_bde9_5361_9460_f8d7_338b_ccd9
        );
        // Just below the swap boundary √2/2.
        assert_eq!(
            asinq(consts::FRAC_1_SQRT_2).to_bits(),
            0x3ffe_921f_b544_42d1_8469_898c_c517_01b8
        );
        assert_eq!(
            asinq(1.0 - crate::f128_::exp2i(-113)).to_bits(),
            0x3fff_921f_b544_42d1_8369_898c_c517_01b8
        );

        assert_eq!(acosq(0.5).to_bits(), consts::FRAC_PI_3.to_bits());
        assert_eq!(
            acosq(-0.5).to_bits(),
            0x4000_0c15_2382_d736_5846_5bb3_2e0f_567b
        );
        assert_eq!(
            acosq(0.75).to_bits(),
            0x3ffe_720a_392c_1d95_4851_badb_d6cd_2d8e
        );
        assert_eq!(
            acosq(-0.75).to_bits(),
            0x4000_359d_26f9_3b6c_3255_1ad5_cf63_b655
        );
        // acos just below 1 lands exactly on 2^-56.
        assert_eq!(
            acosq(1.0 - crate::f128_::exp2i(-113)).to_bits(),
            0x3fc7_0000_0000_0000_0000_0000_0000_0000
        );
        // A tiny argument leaves π/2 untouched in both directions.
        let tiny = crate::f128_::exp2i(-120);
        assert_eq!(acosq(tiny).to_bits(), consts::FRAC_PI_2.to_bits());
        assert_eq!(acosq(-tiny).to_bits(), consts::FRAC_PI_2.to_bits());
    }

    #[test]
    fn tiny_and_subnormal() {
        // asin(x) − x = x³/6·(1 + …) < ½ ulp for every |x| < 2^-56.
        for x in [
            f128::from_bits(1),
            f128::MIN_POSITIVE,
            crate::f128_::exp2i(-57),
            crate::f128_::exp2i(-56),
            f128::from_bits(0x3fc6_dead_beef_0123_4567_89ab_cdef_0123),
        ] {
            assert_eq!(asinq(x).to_bits(), x.to_bits(), "{:#x}", x.to_bits());
            assert_eq!(asinq(-x).to_bits(), (-x).to_bits());
        }
    }
}

/// MPFR certification that `atan2q`'s [`ZIV_GATE`] still covers the fast
/// leg's true error — now including the wide root's slip and the top-128
/// truncation of the 256-bit reduction — with the 2× margin the project
/// requires.  Run with
/// `CC=clang cargo +nightly test --release --features "f128 mpfr"`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::super::MANTISSA_MASK;
    use super::super::atan2::ZIV_GATE;
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

    /// A positive magnitude strictly inside `(0, 1)`.
    fn sample(i: u64) -> f128 {
        let bits = mix128(i);
        // Everything from just above zero to just below one, weighted toward
        // the top binades where the reduction and the root work hardest.
        let exponent = match bits >> 112 & 7 {
            0..4 => 0x3ffe,
            4..6 => 0x3ffe - (bits >> 115) % 8,
            6 => 0x3ffe - (bits >> 115) % 64,
            _ => 1 + (bits >> 115) % 0x3ffe,
        };
        f128::from_bits(exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// A few ulps around the breakpoint pullbacks `sin(atan(i/64))` and
    /// `cos(atan(i/64))`, where the reduced tangent collapses.
    fn near_breakpoint(i: u64) -> f128 {
        let bits = mix128(i);
        let k = (bits >> 113 & 63) + 1;
        let t = k as f128 / 64.0;
        let c = super::super::roots::rsqrtq(crate::f128_::fma128(t, t, 1.0));
        let x = if bits & (1 << 126) == 0 { t * c } else { c };
        f128::from_bits(x.to_bits().wrapping_add(bits >> 119 & 15).wrapping_sub(7))
    }

    /// `|frame − truth| / ZIV_GATE` in the gate's own units of the fast
    /// frame's 2^(e2−128).
    fn slip(x: f128, acos: bool, xneg: bool) -> f64 {
        let (m, e) = split(x.to_bits());
        let sq = wide_sqrt(m, e);
        let r = reduce(m << 15, e + 1, &sq, acos, xneg);
        let (frac, e2) = fast(&r);
        let frame = Float::with_val(PRECISION, frac) * Float::with_val(PRECISION, 2).pow(e2 - 128);
        let arg = Float::with_val(PRECISION, if xneg { -x } else { x });
        let truth = if acos { arg.acos() } else { arg.asin() };
        let unit: Float = Float::with_val(PRECISION, 2).pow(e2 - 128);

        (Float::with_val(PRECISION, truth - frame).abs() / unit).to_f64() / ZIV_GATE as f64
    }

    /// Worst `|err|/gate` over the sample, all four function/sign flavors.
    #[test]
    fn fast_leg_is_sound() {
        let mut worst = 0.0;
        let mut worst_at = (0.0_f128, false, false);

        for i in 0..SAMPLES {
            let x = if i % 4 == 0 {
                near_breakpoint(i)
            } else {
                sample(i)
            };

            for (acos, xneg) in [(false, false), (true, false), (true, true)] {
                let ratio = slip(x, acos, xneg);

                if ratio > worst {
                    worst = ratio;
                    worst_at = (x, acos, xneg);
                }
            }
        }
        println!("asinq/acosq fast leg: worst |err|/gate = {worst:.4} at {worst_at:?}");
        assert!(
            worst < 0.5,
            "asinq/acosq gate covers only {:.2}× the slip at {worst_at:?}",
            1.0 / worst
        );
    }
}
