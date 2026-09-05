//! The binary128 arc sine and arc cosine.
//!
//! Two legs meet at [`BAND`].  Below `|x| = 2^-3` the arc sine *is* its own
//! reduced argument: no square root is formed at all and the fast leg is the
//! bare Taylor series ([`series`]), in floating form for `asin` and summed
//! into `π/2 ∓ ·` in `atan2q`'s frame for `acos`.  The band pays for its own
//! width one binade at a time: [`taylor_14`] serves everything below 2^-4,
//! and only the top binade runs the nineteen terms of [`taylor_19`].  Since
//! the root pipeline costs some 2.7× the series even there, every binade the
//! series can reach is one the root should not.
//!
//! Above it the fast leg ([`root_frame`]) reduces on *dyadic sine
//! breakpoints* and never divides.  With `u = min(x, √(1−x²))` and
//! `v = max(x, √(1−x²))` — so `u ≤ √2/2`, and `asin(x)` is `asin(u)` or
//! `π/2 − asin(u)` — the breakpoint is `sin φ_j = j/128`, `j = round(128·u)`
//! read straight off `u`'s top bits, and the sine difference
//!
//! ```text
//! t = sin(asin(u) − φ_j) = u·cos φ_j − v·(j/128)
//! ```
//!
//! makes one side an *exact* small-integer product and the other a 256-bit
//! multiply by the tabulated `cos φ_j` ([`COS`]).  `|t| < 2^-7.4`, so the
//! same Taylor series as the band below ([`series`]) finishes it, and
//! `asin(u) = φ_j + asin(t)` sums with [`PHI`] in `atan2q`'s 2^-253 frame.
//! The pieces, in order:
//!
//! 1. **Square.** `1 − x²` is *exact* at 2^-256: `x²` is a 226-bit integer
//!    and the subtraction from 1 cancels without error.  Since
//!    `|x| ≤ 1 − 2^-113`, the difference stays above 2^-112 and the root
//!    above 2^-57.
//! 2. **Root.** [`sqrt_wide`](super::roots::sqrt_wide)'s pure-integer frame
//!    — a Taylor-table seed `r ≈ 1/(2√z)` and its series correction — reads
//!    the top 128 bits of the normalized `1 − x²` to ~2^-121; one Newton step
//!    against all 256 bits, dividing by the *same* seed, lands ~2^-160
//!    ([`root_256`]).  Absolute accuracy is all the difference needs: `t`'s
//!    error enters the result directly, against a half-ulp of at least 2^-116.
//! 3. **Reduce and sum.** The difference, its normalization, the series, and
//!    the table sum; the whole leg then rides `atan2q`'s guard and certified
//!    Ziv gate.  When `v` is `x`, its product is exact and `u`'s 2^sh scale
//!    (the root's normalization parity) rides through the frame unshifted.
//!    A zero sector — `√(1−x²) < 2^-8`, i.e. `x` within 2^-17 of 1 — is the
//!    relative band: the series in floating form on the root itself.
//!    [`ziv_soundness`] certifies the gate across both legs.
//! 4. **Refine.** The accurate leg is the [`atan2q`](super::atan2) pipeline
//!    fed a 384-bit root: `asin(x) = atan2(x, √(1−x²))`, with an f64-seeded
//!    reciprocal square root refined by one of [`hypotq`](super::hypot)'s
//!    doubling steps and two Newton steps ([`wide_sqrt`], [`sqrt_384`]), a
//!    384-bit dyadic-tangent reduction, a Newton reciprocal of the wide
//!    denominator ([`recip_wide`]), and `atan2q`'s own [`atan_frac_384`],
//!    tables, and rounder.  It also catches every gate miss from the series
//!    band, where it forms the root it skipped.
//!
//! Tiny arguments need no special path either way: once `x² < 2^-128` of `x`
//! the series is exactly `x`, which is what `asin(x) = x + x³/6 + …` rounds
//! to, all the way down through the subnormals.

use super::asin_tables::{COS, PHI};
use super::atan2::{
    add_signed_256, assemble_384, atan_frac_384, combine, combine_phi, place, recip_128, round_384,
    round_fast, shr_round, top_256,
};
use super::hypot::rsqrt_step;
use super::roots::sqrt_wide_seeded;
use super::uint::{
    add_256, add_384, cmp_384, leading_zeros_256, leading_zeros_384, mhi, mhi_approx, mul_hi_64,
    mul_hi_384, neg_384, shl_256, shl_384, shr_256_sat, shr_384_sat, sub_256, sub_384, wmul,
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

/// Frame exponent below which the fast leg is the bare Taylor series: `x` is
/// its own reduced argument and no square root is formed at all.
///
/// A binade moved in here costs the series terms and saves it a whole root —
/// 63 ns against 172 at this edge, so the trade is worth making as long as the
/// terms are charged only to the binade that needs them ([`NARROW_BAND`]).
/// One binade further would need some twenty-nine terms, which is where the
/// two finally meet.
const BAND: i32 = -3;

/// Frame exponent below which the series needs only [`taylor_14`]: the wider
/// chain of [`taylor_19`] is the top binade's alone.
const NARROW_BAND: i32 = -4;

/// `asin(x)/x = Σ_k A_k·x^(2k)` past its leading 1, in units of 2^-128:
/// `A_k = binomial(2k+2, k+1)/(4^(k+1)·(2k+3))`, the arc sine's own Taylor
/// coefficients, rounded to nearest.  Nineteen of them cover the whole band:
/// the twentieth term is 2^-128.4 at `|x| = 2^-3`, and [`taylor_14`] drops
/// the last five because at 2^-4 the fifteenth is already 2^-127.7.
///
/// ```text
/// python3 -c 'from fractions import Fraction as F
/// from math import comb
/// for k in range(1, 20):
///     q = F(comb(2*k, k), 4**k*(2*k+1)) * (1 << 128)
///     h = f"{(2*q.numerator + q.denominator)//(2*q.denominator):032x}"
///     print("0x" + "_".join(h[i:i+4] for i in range(0, 32, 4)) + ",")'
/// ```
const COEF: [u128; 19] = [
    0x2aaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaab,
    0x1333_3333_3333_3333_3333_3333_3333_3333,
    0x0b6d_b6db_6db6_db6d_b6db_6db6_db6d_b6db,
    0x07c7_1c71_c71c_71c7_1c71_c71c_71c7_1c72,
    0x05ba_2e8b_a2e8_ba2e_8ba2_e8ba_2e8b_a2e9,
    0x0471_3b13_b13b_13b1_3b13_b13b_13b1_3b14,
    0x0393_3333_3333_3333_3333_3333_3333_3333,
    0x02f5_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f,
    0x027f_bca1_af28_6bca_1af2_86bc_a1af_286c,
    0x0225_de79_e79e_79e7_9e79_e79e_79e7_9e7a,
    0x01df_3bd3_7a6f_4de9_bd37_a6f4_de9b_d37a,
    0x01a6_863d_70a3_d70a_3d70_a3d7_0a3d_70a4,
    0x0178_2dda_12f6_84bd_a12f_684b_da12_f685,
    0x0151_ba30_8d3d_cb08_d3dc_b08d_3dcb_08d4,
    0x0131_683b_def7_bdef_7bde_f7bd_ef7b_def8,
    0x0115_ee9d_45d1_745d_1745_d174_5d17_45d1,
    0x00fe_57c7_db6d_b6db_6db6_db6d_b6db_6db7,
    0x00e9_e954_706e_b3e4_5306_eb3e_4530_6eb4,
    0x00d8_137a_bd89_d89d_89d8_9d89_d89d_89d9,
];

/// The shared pipeline for `0 < |x| < 1`: `x = m·2^(e−112)`, with `sign`
/// applied to the result (asin only) and `xneg` steering the acos quadrant.
fn arc(m: u128, e: i32, sign: u128, acos: bool, xneg: bool) -> f128 {
    let (frac, e2) = fast_frame(m, e, acos, xneg);

    round_fast(frac, e2, sign)
        .unwrap_or_else(|| accurate(m << 15, e + 1, &wide_sqrt(m, e), acos, xneg, sign))
}

/// The fast leg's floating frame, shared with [`ziv_soundness`].
///
/// Below [`BAND`] the root would be a wasted 2^-207 approximation of 1: `x` is
/// its own reduced argument, so the series is the whole leg — in floating form
/// for `asin`, summed into `π/2 ∓ ·` in `atan2q`'s 2^-253 frame for `acos`.
#[inline]
fn fast_frame(m: u128, e: i32, acos: bool, xneg: bool) -> (u128, i32) {
    let fx = m << 15;
    let ex = e + 1;

    if ex <= BAND {
        let (f, et) = series(fx, ex, |v| {
            if ex <= NARROW_BAND {
                taylor_14(v)
            } else {
                taylor_19(v)
            }
        });

        return if acos {
            combine(place(f, et), 0, false, 1, !xneg)
        } else {
            (f, et)
        };
    }
    root_frame(fx >> ex.unsigned_abs(), e, acos, xneg)
}

/// The significand of `√2/2` rounded to nearest: `x` above it (in the binade
/// `[½, 1)`) makes the root the smaller side of the sort.
const M_FRAC_1_SQRT_2: u128 = 0x1_6a09_e667_f3bc_c908_b2fb_1366_ea95;

/// Log2 of the breakpoint denominator: `sin φ_j = j/128`.
const DENOM_BITS: u32 = 7;

/// The root band's fast frame: `asin(x)` (or `acos`) as a floating 128-bit
/// fraction for `2^-3 ≤ |x| < 1`, by the sine-difference reduction described
/// in the module docs.  `xs = x·2^128` exactly (the band's three binades all
/// fit: the significand has fifteen spare bits), `x = m·2^(e−112)`.
#[inline]
fn root_frame(xs: u128, e: i32, acos: bool, xneg: bool) -> (u128, i32) {
    let (s, sh) = root_256(xs);
    // `x > √2/2` makes the root the smaller side; the constant compare keeps
    // the sort off the root's critical path.  Then `x` sits in `[½, 1)`, so
    // `xs` *is* `x·2^256`'s top limb; otherwise the root is at least `√2/2`
    // and `sh = 0`, so `u` carries no scale at all.
    let x_big = (e == -1) & (xs > M_FRAC_1_SQRT_2 << 15);
    // By mask, not branch: a fifth of the band's draws take the other arm,
    // and LLVM turned the four-limb select into a jump.
    let mask = 0u128.wrapping_sub(u128::from(x_big));
    let u = [s[0] & mask, (s[1] & mask) | (xs & !mask)];
    let v = [s[0] & !mask, (xs & mask) | (s[1] & !mask)];
    let (quadrant, negate) = plumbing(x_big != acos, xneg);

    // `j = round(128·u)` from the top 64 bits of `u·2^(256+sh)`: the dropped
    // bits cannot carry across a multiple of 2^(57+sh), so the rounding is
    // exact.  `sh ≥ 8` is `u < 2^-8`, which rounds to the sectorless band.
    let j = if sh >= 8 {
        0
    } else {
        ((u[1] >> 64) + (1 << (56 + sh))) >> (57 + sh)
    } as usize;
    debug_assert!(j < COS.len());

    let (t1, et, negative, phi) = if j == 0 {
        // The relative band: only the root gets here (`x ≥ 2^-3` rounds to a
        // sector), and it is its own reduced argument.  Its top bit can have
        // slipped below 2^255 by the Newton step's few units.
        let lz = s[1].leading_zeros();
        (top_256(s, lz), -(sh as i32) - lz as i32, false, [0, 0])
    } else {
        // `t·2^(256+sh) = u·cos φ_j − v·(j·2^sh/128)`, both sides below 2^255.5:
        // `j·2^sh ≤ 128` because `j ≤ 2^(7−sh)`, so the exact side is a 7-bit
        // multiple of `v` cut down by seven bits first (under 2^14 units of
        // slip).
        let a = mul_hi_192(u, COS[j]);
        let k = (j as u128) << sh;
        let v7 = [
            (v[0] >> DENOM_BITS) | (v[1] << (128 - DENOM_BITS)),
            v[1] >> DENOM_BITS,
        ];
        let (bh, bl) = wmul(v7[0], k);
        let b = [bl, v7[1] * k + bh];
        let (mag, negative) = sub_abs_256(a, b);
        // `|t| ≤ 2^-7.4`, but it collapses toward zero at a breakpoint: the
        // normalization is a full 256-bit one, past `et ≤ −64` the series is
        // `t` itself, and an exact zero (clamped to a 255-bit shift) places as
        // nothing at all.
        let lz = leading_zeros_256(mag).min(255);
        (
            shl_256(mag, lz)[1],
            -(lz as i32) - sh as i32,
            negative,
            PHI[j],
        )
    };
    let (f, et) = series(t1, et, taylor_7);

    combine_phi(phi, place(f, et), negative, quadrant, negate)
}

/// `√(1 − x²)` for the fast leg from `xs = x·2^128`: `(S, sh)` with `√(1−x²) = S·2^-256·2^-sh`,
/// `S` within ~2^95 units of the truth and `sh` the root's normalization
/// (even shift halved), so `S ≈ 2^256·√z` for a `z ∈ [¼, 1)` — nominally in
/// `[2^255, 2^256)`, a few units below it at worst.
///
/// `1 − x²` is exact; [`sqrt_wide_seeded`] reads its top 128 bits to ~2^-121
/// and hands back the seed `r ≈ 1/(2√z)` it divided by, so one Newton step
/// `s + (z − s²)·r` against all 256 bits reaches `ε²/2 + ε·2^-42 ≈ 2^-162`.
#[inline]
fn root_256(xs: u128) -> ([u128; 2], u32) {
    // `xs = x·2^128` exactly, so its square is `x²·2^256` with no shift at all.
    let (hi, lo) = wmul(xs, xs);
    let v = sub_256([0, 0], [lo, hi]);
    // 1 − x² ≥ 2^-112·(1 − 2^-114) (at `x = 1 − 2^-113`) keeps at most 112
    // leading zeros.
    let lz = leading_zeros_256(v);
    debug_assert!(lz <= 112);
    let parity = lz & !1;
    let vn = shl_256(v, parity);

    // s0 = √(4z)·2^125 = √z·2^126 from the top limb alone; r·2^-63 = 1/(2√z).
    let (s0, r) = sqrt_wide_seeded(vn[1]);
    let (qh, ql) = wmul(s0, s0);
    let (res, downward) = sub_abs_256(vn, shl_256([ql, qh], 4));
    // |z − s0²| < 2^-119 leaves the residual under 2^137 in the frame.
    debug_assert!(res[1] < 1 << 12);
    let top = (res[1] << 116) | (res[0] >> 12);
    let (ph, pl) = wmul(top, u128::from(r));
    // The correction |z − s0²|·r·2^-63, i.e. the product back down by 51.
    let corr = [(pl >> 51) | (ph << 77), ph >> 51];
    let s = add_signed_256([0, s0 << 2], corr, downward);

    (s, parity >> 1)
}

/// `⌊u·c / 2^256⌋` for 256-bit `u`, `c`, from their top 192 bits each: the six
/// 64-bit products at or above the window's low edge, so the result runs
/// under 2^66 units short of the truth — nothing against the root band's
/// 2^116-unit budget, where a full 256-bit high product is sixteen `mulx`.
#[inline]
fn mul_hi_192(u: [u128; 2], c: [u128; 2]) -> [u128; 2] {
    let (u3, u2, u1) = (u[1] >> 64, u[1] & LOW, u[0] >> 64);
    let (c3, c2, c1) = (c[1] >> 64, c[1] & LOW, c[0] >> 64);
    // Weight 2^256 of `u·c` — the window's unit — from three products, and
    // weight 2^320 from two, with every carry chained into the top limb.
    let (s0, k0) = (u2 * c2).overflowing_add(u3 * c1);
    let (s0, k1) = s0.overflowing_add(u1 * c3);
    let (m, km) = (u3 * c2).overflowing_add(u2 * c3);
    let (t, kt) = m.overflowing_add(s0 >> 64);
    let (t, kt2) = t.overflowing_add((u128::from(k0) + u128::from(k1)) << 64);
    let carries = u128::from(km) + u128::from(kt) + u128::from(kt2);

    [
        (t << 64) | (s0 & LOW),
        u3 * c3 + (t >> 64) + (carries << 64),
    ]
}

/// The low 64 bits of a limb.
const LOW: u128 = u64::MAX as u128;

/// `(|a − b|, a < b)` at 256 bits without a data-dependent branch: the
/// borrow out of the wrapping difference *is* the comparison (a short-circuit
/// `>` chain would compile to two coin-flip jumps), and it selects a
/// two's-complement negation by mask.
#[inline]
fn sub_abs_256(a: [u128; 2], b: [u128; 2]) -> ([u128; 2], bool) {
    let (low, borrow) = a[0].overflowing_sub(b[0]);
    let (high, negative) = a[1].overflowing_sub(b[1]);
    let (high, chained) = high.overflowing_sub(u128::from(borrow));
    let negative = negative | chained;
    let d = [low, high];
    let mask = 0u128.wrapping_sub(u128::from(negative));

    (
        add_256([d[0] ^ mask, d[1] ^ mask], [u128::from(negative), 0]),
        negative,
    )
}

/// `asin(x)` as a floating 128-bit fraction from `x = t1·2^(et−128)`, for
/// `et ≤ BAND`.
///
/// The reduced argument *is* `x`, so the whole reduction collapses to the
/// series: with `u = x²` rounded into a 2^-128 word and `v = u²`, the even and
/// odd halves of `Σ A_k·u^k` are two independent Horner chains in `v` — half
/// the serial depth — and `x³` multiplies in while they run.  Each half closes
/// on a 64-bit tail whose slack sits far below `Q`; the correction itself is
/// only 2^-8.6 of the result, so `Q` never needs more than 113 bits.
///
/// The chains below are the same series truncated per band: `taylor` maps
/// `v = x⁴` to the even and odd halves.  Only the series band's top binade
/// needs nineteen terms (and full-width coefficients out to `A_10`);
/// everything under 2^-4 is served by [`taylor_14`] at five terms and three
/// levels less, which is the whole reason the band is split rather than run
/// wide throughout, and the root band's `|t| < 2^-7.4` by [`taylor_7`].
#[inline]
fn series(t1: u128, et: i32, taylor: impl Fn(u128) -> (u128, u128)) -> (u128, i32) {
    let sh = (-2 * et) as u32;
    // `x² < 2^-128` of `x`: `asin(x) − x < ½ulp`, and the series is exactly `x`.
    if sh >= 128 {
        return (t1, et);
    }
    let u = shr_round(mhi_approx(t1, t1), sh);
    let cube = mhi_approx(t1, u);
    let v = mhi_approx(u, u);
    let (even, odd) = taylor(v);
    let (f, carry) = t1.overflowing_add(mhi_approx(cube, even + mhi_approx(u, odd)));

    // `asin(x)/x < 1 + 2^-8.6` carries out of the frame only just below 2^128.
    if carry {
        ((f >> 1) | 1 << 127, et + 1)
    } else {
        (f, et)
    }
}

/// The even and odd halves of `Σ A_k·v^k` for `|x| < 2^-7.4`, the root band's
/// reduced argument: seven terms, the last of each half 64-bit.  The dropped
/// eighth is under 2^-132 of `x` there (`A_8·x^16`), against an absolute
/// half-ulp of at least 2^-116 in the root band and a relative 2^-113 in the
/// band below 2^-8; a 64-bit `A_5` slips `2^-64·x^10 < 2^-138` of `x`, which
/// only the relative band can even see.
fn taylor_7(v: u128) -> (u128, u128) {
    let narrow = |k: usize| u128::from((COEF[k] >> 64) as u64) << 64;
    let even = COEF[0]
        + mhi_approx(
            v,
            COEF[2] + mhi_approx(v, COEF[4] + mhi_approx(v, narrow(6))),
        );
    let odd = COEF[1] + mhi_approx(v, COEF[3] + mhi_approx(v, narrow(5)));

    (even, odd)
}

/// The even and odd halves of `Σ A_k·v^k` for `|x| < 2^-4`: fourteen terms,
/// the last seven of them 64-bit — `x^14` already buries a half coefficient's
/// own slack below `Q` there.
fn taylor_14(v: u128) -> (u128, u128) {
    let vh = (v >> 64) as u64;
    let narrow = |k: usize| (COEF[k] >> 64) as u64;
    let tail_even = narrow(8) + mul_hi_64(vh, narrow(10) + mul_hi_64(vh, narrow(12)));
    let tail_odd = narrow(7)
        + mul_hi_64(
            vh,
            narrow(9) + mul_hi_64(vh, narrow(11) + mul_hi_64(vh, narrow(13))),
        );
    let even = COEF[0]
        + mhi_approx(
            v,
            COEF[2]
                + mhi_approx(
                    v,
                    COEF[4] + mhi_approx(v, COEF[6] + mhi_approx(v, u128::from(tail_even) << 64)),
                ),
        );
    let odd = COEF[1]
        + mhi_approx(
            v,
            COEF[3] + mhi_approx(v, COEF[5] + mhi_approx(v, u128::from(tail_odd) << 64)),
        );

    (even, odd)
}

/// [`taylor_14`] for the band's top binade, `2^-4 ≤ |x| < 2^-3`: nineteen
/// terms, and the 64-bit tails cannot start before `A_11` — at `x = 2^-3` a
/// half `A_8` would slip 2^11 units of the frame, where a half `A_11` slips
/// 2^-7.
fn taylor_19(v: u128) -> (u128, u128) {
    let vh = (v >> 64) as u64;
    let narrow = |k: usize| (COEF[k] >> 64) as u64;
    let tail_even = narrow(10)
        + mul_hi_64(
            vh,
            narrow(12)
                + mul_hi_64(
                    vh,
                    narrow(14) + mul_hi_64(vh, narrow(16) + mul_hi_64(vh, narrow(18))),
                ),
        );
    let tail_odd = narrow(11)
        + mul_hi_64(
            vh,
            narrow(13) + mul_hi_64(vh, narrow(15) + mul_hi_64(vh, narrow(17))),
        );
    let even = COEF[0]
        + mhi_approx(
            v,
            COEF[2]
                + mhi_approx(
                    v,
                    COEF[4]
                        + mhi_approx(
                            v,
                            COEF[6]
                                + mhi_approx(
                                    v,
                                    COEF[8] + mhi_approx(v, u128::from(tail_even) << 64),
                                ),
                        ),
                ),
        );
    let odd = COEF[1]
        + mhi_approx(
            v,
            COEF[3]
                + mhi_approx(
                    v,
                    COEF[5]
                        + mhi_approx(
                            v,
                            COEF[7]
                                + mhi_approx(
                                    v,
                                    COEF[9] + mhi_approx(v, u128::from(tail_odd) << 64),
                                ),
                        ),
                ),
        );

    (even, odd)
}

/// `√(1 − x²)` carried wide, plus the pieces the accurate leg reuses.
struct Sqrt {
    /// `(1 − x²)·2^(384+2·shift)`, normalized into `[2^382, 2^384)` by an even
    /// shift, with the square ⌈·⌉ed at 2^-384 so the value never overshoots.
    vn: [u128; 3],
    /// `≈ 2^190/√vn[2]`, the reciprocal square root both Newton steps divide
    /// by, good to ~2^-102.
    q: u128,
    /// `√(vn·2^-384)·2^256` unnormalized (clamped to all-ones at the top),
    /// within ~2^-207 relative after one Newton step.
    frame: [u128; 2],
    /// `√(1−x²) = (frame·2^-256)·2^es_base`.
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

    // `hypotq`'s exact-tier seed, but only one of its doubling steps: the
    // Newton step below squares 2^-102 into 2^-207, and the fast leg's true
    // need is 2^-133 — the sector's 2^7 cancellation over a 2^-126 frame.
    let seed = crate::exp2i(158) / ((w >> 64) as u64 as f64).sqrt();
    let q = rsqrt_step(w, seed as u128);

    // s0h ≈ √(vn·2^-384)·2^128; the shift clamps when the root grazes 1.
    let s0 = mhi(w, q);
    let s0h = if s0 >> 126 == 0 { s0 << 2 } else { u128::MAX };

    // One Newton step: frame = s0h·2^128 + (vtop − s0h²)·q/2^127, since
    // q·2^-127 ≈ 1/(2√z) in the frame's own units.
    let (sqh, sql) = wmul(s0h, s0h);
    let (h, downward) = sub_signed_256(&[vn[1], vn[2]], &[sql, sqh]);
    debug_assert!(h[1] < 1 << 34);
    let hs = (h[1] << 94) | (h[0] >> 34);
    let (p1, p0) = wmul(hs, q);
    let corr = [(p1 << 35) | (p0 >> 93), p1 >> 93];
    let frame = if downward {
        sub_256([0, s0h], corr)
    } else {
        let sum = add_256([0, s0h], corr);
        if sum[1] < s0h { [u128::MAX; 2] } else { sum }
    };

    debug_assert!(leading_zeros_256(frame) <= 1);
    #[allow(clippy::cast_possible_wrap)]
    let es_base = -((parity / 2) as i32);

    Sqrt {
        vn,
        q,
        frame,
        es_base,
    }
}

/// One more Newton step against the full 384-bit `1 − x²`, to ~2^-305:
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
    debug_assert!(h[1] < 1 << 96 && h[2] == 0);
    let hs = (h[1] << 32) | (h[0] >> 96);
    let (p1, p0) = wmul(hs, sq.q);
    let corr = [(p1 << 97) | (p0 >> 31), p1 >> 31, 0];
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
    fn mul_hi_192_runs_short_by_under_2_66() {
        use super::super::uint::{mul_hi_256, sub_256};
        let mut z = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = || {
            z ^= z << 13;
            z ^= z >> 7;
            z ^= z << 17;
            u128::from(z) << 64 | u128::from(z.wrapping_mul(0x2545_F491_4F6C_DD1D))
        };
        for _ in 0..100_000 {
            let u = [next(), next()];
            let c = [next(), next()];
            let exact = mul_hi_256(u, c);
            let short = sub_256(exact, mul_hi_192(u, c));
            assert_eq!(short[1], 0);
            assert!(short[0] < 1 << 66, "{u:x?} {c:x?} {short:x?}");
        }
    }

    /// The root band's Ziv fallback rate: the gate is 2^7 of 2^15 guard
    /// positions, so about 0.4% of uniform draws should refuse.
    #[test]
    fn root_band_fallback_rate() {
        let mut z = 0x2545_F491_4F6C_DD1D_u64;
        let mut next = || {
            z ^= z << 13;
            z ^= z >> 7;
            z ^= z << 17;
            z
        };
        let mut refused = 0;
        const N: u32 = 1_000_000;
        for _ in 0..N {
            let bits = u128::from(next()) << 64 | u128::from(next());
            let exponent = 0x3ffc + (bits >> 112) % 3;
            let (m, e) = split(exponent << EXP_SHIFT | bits & super::super::MANTISSA_MASK);
            let (frac, e2) = fast_frame(m, e, false, false);
            refused += u32::from(round_fast(frac, e2, 0).is_none());
        }
        println!("root band fallback: {refused} of {N}");
        assert!(refused < N / 100);
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
            0..3 => 0x3ffe,
            3..5 => 0x3ffe - (bits >> 115) % 8,
            5 => 0x3ffe - (bits >> 115) % 64,
            // The series band's own top binades, where its truncated Taylor
            // tail is worst; the arm below reaches the rest, subnormals
            // included.
            6 => 0x3ffc - (bits >> 115) % 4,
            _ => 1 + (bits >> 115) % 0x3ffe,
        };
        f128::from_bits(exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// A few ulps around the breakpoints `j/128` and their pullbacks
    /// `√(1 − (j/128)²)`, where the sine difference collapses and the root's
    /// slip is all that is left of `t`.
    fn near_breakpoint(i: u64) -> f128 {
        let bits = mix128(i);
        let j = (bits >> 113 & 127) % (COS.len() as u128 - 1) + 1;
        let t = j as f128 / 128.0;
        let x = if bits & (1 << 126) == 0 {
            t
        } else {
            super::super::roots::sqrtq(crate::f128_::fma128(-t, t, 1.0))
        };
        f128::from_bits(x.to_bits().wrapping_add(bits >> 119 & 15).wrapping_sub(7))
    }

    /// `|frame − truth| / ZIV_GATE` in the gate's own units of the fast
    /// frame's 2^(e2−128).
    fn slip(x: f128, acos: bool, xneg: bool) -> f64 {
        let (m, e) = split(x.to_bits());
        let (frac, e2) = fast_frame(m, e, acos, xneg);
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

/// The fast leg's root, checked through the residual [`sqrt_384`] already
/// forms: `|vn − frame²|` reaching 50 bits above the low limb puts `frame`
/// within 2^-207 of `√vn`, far inside the 2^-133 the sector's 2^7 cancellation
/// over a 2^-126 reduction actually needs, and leaves that step's own 96-bit
/// window 46 bits clear.  A shift constant cut too fine in [`wide_sqrt`]
/// corrupts the frame and widens this residual first.
#[cfg(test)]
mod frame_residual {
    use super::super::MANTISSA_MASK;
    use super::*;

    #[test]
    fn newton_step_has_headroom() {
        let mut worst = 0;

        for i in 0..2_000_000u64 {
            let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            let bits =
                u128::from(z ^ (z >> 31)) << 64 | u128::from(i.wrapping_mul(0x9E37_79B9_7F4A_7C15));
            // Biased 1..=0x3ffe: every `|x| < 1`, the only inputs `asinq` hands `wide_sqrt`.
            let exponent = 1 + (bits >> 112) % 0x3ffe;
            let (m, e) = split(exponent << EXP_SHIFT | bits & MANTISSA_MASK);
            let sq = wide_sqrt(m, e);
            let square = square_top(sq.frame);
            let h = if cmp_384(sq.vn, square).is_lt() {
                sub_384(square, sq.vn)
            } else {
                sub_384(sq.vn, square)
            };
            assert_eq!(h[2], 0);
            worst = worst.max(128 - h[1].leading_zeros());
        }
        println!("wide_sqrt residual reaches {worst} bits of a 96-bit window");
        assert!(worst < 96);
    }
}
