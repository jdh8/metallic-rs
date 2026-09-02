//! The binary128 sine and cosine.
//!
//! Both are one pipeline on `|x|` — the sine is odd and the cosine even, so
//! the sign of `x` is peeled off and, for the sine, put back at the end:
//!
//! 1. **Reduce.** `x·2/π` is formed as an exact product of the 113-bit
//!    significand with a window of 2/π cut from 64-bit limbs at the exponent
//!    (Payne–Hanek): five limbs on the fast leg, nine on the accurate one.
//!    Everything above the units bit but its two low bits is a multiple of 4
//!    and drops; what is left is the quadrant and a fraction, 192 bits wide on
//!    the fast leg and 448 on the accurate one.
//! 2. **Split.** Quadrant and fraction round to `n = round(512·x/π) mod 512`
//!    — the quadrant `n >> 7` and a breakpoint `j = n & 127` — leaving a
//!    signed residual `g` with `|g| ≤ 1/256`, so `θ = g·π/2` stays within
//!    `π/512 < 2^-7.35`.  `|g|` normalizes into a floating fraction at its
//!    own exponent, so the residual keeps full relative precision however
//!    deep the cancellation; the fast leg only insists on 136 of its 192 bits
//!    ([`MAX_LZ`]) and hands anything closer to a multiple of π/2 over.
//! 3. **Evaluate.** `sin θ = θ·(1 − u·(A − u·B))` and `1 − cos θ = u·(½ −
//!    u·(E − u·O))` for `u = θ²`, with `A, B, E, O` the even and odd halves
//!    of the Taylor sums in `v = u²`: six terms each at the fast width,
//!    eighteen at the accurate one, in the family's tiered fixed point.
//! 4. **Recombine.** With `A = j·π/256 + θ`, `sin |x|` is `±sin A` or
//!    `±cos A` by quadrant, and `sin A = S_j·cos θ + C_j·sin θ`, `cos A =
//!    C_j·cos θ − S_j·sin θ` from tables of `sin(j·π/256)` and `cos(j·π/256)`:
//!    two products into a frame at 2^-256 (fast) or 2^-384 (accurate).
//!    `sin A ≥ sin(π/512)` and `cos A ≥ cos(257π/512)` keep the frame normal
//!    unless `j = 0` and the sine is wanted — then `sin A = sin θ` keeps its
//!    floating form the whole way, like `atan2q`'s sectorless band.
//! 5. **Round.** `atan2q`'s rounders: the fast leg on a fixed 15-bit guard,
//!    the tie window ([`ZIV_GATE`](super::atan2::ZIV_GATE)) to the accurate
//!    leg, which decides on 384
//!    bits.
//!
//! Below `|x| = 2^-8` ([`DIRECT`]) there is nothing to reduce: `θ = x`
//! exactly, and the series alone is the fast leg.  Below 2^-57 ([`TINY`]) the
//! cubic and quadratic terms sit under half an ulp, so `sin x = x` and
//! `cos x = 1` outright, subnormals and zero included.

use super::atan2::{add_signed_256, place_256, round_384, round_fast, shr_round, top_256};
use super::trig_tables::{COS_COEF, FRAC_2_PI, PIO2_128, PIO2_384, SIN_COEF, SINCOS};
use super::uint::{
    add_384, funnel_down, leading_zeros_384, mhi_approx, mul_hi_384, shl_384, shr_384_sat, sub_256,
    sub_384, wmul,
};
use super::{BIAS, EXP_MASK, EXP_SHIFT, QUIET_BIT, SIGN_MASK, split};

/// Below 2^-57 the sine rounds to `x` and the cosine to 1: `x³/6 < 2^-170.6`
/// sits under the half ulp 2^-171 of `x`, and `x²/2 < 2^-115` under the half
/// ulp 2^-114 below 1.
const TINY: u128 = ((BIAS - 57) as u128) << EXP_SHIFT;

/// Below 2^-8 the argument is its own reduced angle: no window, no table.
const DIRECT: i32 = -8;

/// Leading zeros of the fast leg's 192-bit residual past which it gives up:
/// 136 bits are left at the limit, against a relative need of about 130.
const MAX_LZ: u32 = 56;

/// The sine.
#[must_use]
pub fn sinq(x: f128) -> f128 {
    trig(x, false)
}

/// The cosine.
#[must_use]
pub fn cosq(x: f128) -> f128 {
    trig(x, true)
}

#[inline]
fn trig(x: f128, cosine: bool) -> f128 {
    let bits = x.to_bits();
    let ax = bits & !SIGN_MASK;

    if ax >= EXP_MASK {
        return edge(bits, ax);
    }
    if ax < TINY {
        return if cosine { 1.0 } else { x };
    }
    let sign = if cosine { 0 } else { bits & SIGN_MASK };
    let (m, e) = split(ax);

    fast(m, e, cosine)
        .and_then(|(frac, e2, flip)| round_fast(frac, e2, sign ^ flip))
        .unwrap_or_else(|| accurate(m, e, cosine, sign))
}

/// Infinite or NaN: the former is a domain error, the latter passes through.
#[cold]
#[inline(never)]
fn edge(bits: u128, ax: u128) -> f128 {
    if ax > EXP_MASK {
        f128::from_bits(bits | QUIET_BIT)
    } else {
        f128::NAN
    }
}

/// The 2/π window and the product's alignment for `x = m·2^(e−112)`.
///
/// Bits of 2/π above position `e − 114` contribute multiples of 4 to
/// `x·2/π`, so the window starts at the limb holding bit `e − 113`; within
/// it, `shift` says how far the units bit of `x·2/π` sits below the product's
/// bit `64·W − 2` — negative below `|x| = 2^114`, where the window is the top
/// of 2/π and the integer part is short.
#[inline]
fn window<const W: usize>(e: i32) -> (&'static [u64; W], i32) {
    let big = e - 114;
    let (limb, shift) = if big >= 0 {
        ((big / 64) as usize, big % 64)
    } else {
        (0, big)
    };
    let window = (&FRAC_2_PI[limb..limb + W])
        .try_into()
        .expect("the slice is exactly W limbs long");

    (window, shift)
}

/// `m·win` as `N = W + 2` little-endian 64-bit limbs, for a window `win` of
/// `W` limbs most significant first.
#[inline]
fn product<const W: usize, const N: usize>(m: u128, window: &[u64; W]) -> [u64; N] {
    debug_assert!(N == W + 2);
    let low = m & u128::from(u64::MAX);
    let high = m >> 64;
    let limb = |i: usize| {
        window
            .get(W.wrapping_sub(1).wrapping_sub(i))
            .map_or(0, |&w| u128::from(w))
    };
    let mut p = [0; N];
    let mut carry = 0;

    for (c, out) in p.iter_mut().enumerate() {
        // `high < 2^49` keeps the first sum under 2^114; only the second can
        // spill, and its bit goes straight into the carry.
        let acc = high * limb(c.wrapping_sub(1)) + carry;
        let (acc, spill) = acc.overflowing_add(low * limb(c));
        *out = acc as u64;
        carry = (acc >> 64) | (u128::from(spill) << 64);
    }
    p
}

/// The reduced angle `θ = |g|·π/2` as a floating fraction `t1·2^(et−128)`
/// and the breakpoint index it belongs to.
struct Residual {
    /// `round(512·x/π) mod 512`: quadrant in the top two bits, breakpoint
    /// `j = n & 127` below.
    n: usize,
    /// `g < 0`: the angle sits below its breakpoint.
    negative: bool,
    t1: u128,
    et: i32,
}

/// Payne–Hanek on five limbs of 2/π: `None` when the residual keeps fewer
/// than 136 of the fraction's 192 bits.
#[inline]
fn reduce(m: u128, e: i32) -> Option<Residual> {
    let (window, shift) = window::<5>(e);
    let p: [u64; 8] = product(m, window);
    // The fraction is the 192 bits below the units bit, the quadrant the two
    // above: four funnels at one offset, from limb `i ≤ 3`.
    let base = (126 - shift) as u32;
    debug_assert!((63..=248).contains(&base));
    let i = (base / 64) as usize & 3;
    let bits = base % 64;
    let f0 = funnel_down(p[i], p[i + 1], bits);
    let f1 = funnel_down(p[i + 1], p[i + 2], bits);
    let f2 = funnel_down(p[i + 2], p[i + 3], bits);
    let quadrant = funnel_down(p[i + 3], p[i + 4], bits) & 3;

    // `n = round(512·x/π)`; the residual's top limb becomes signed.
    let top = (u128::from(quadrant) << 64) | u128::from(f2);
    let n = ((top + (1 << 56)) >> 57) as usize & 511;
    let f2 = f2.wrapping_sub((n as u64) << 57);
    let negative = (f2 as i64) < 0;
    let mask = 0u64.wrapping_sub(u64::from(negative));
    let (g0, c0) = (f0 ^ mask).overflowing_add(u64::from(negative));
    let (g1, c1) = (f1 ^ mask).overflowing_add(u64::from(c0));
    let g2 = (f2 ^ mask).wrapping_add(u64::from(c1));
    let lz = g2.leading_zeros();

    if lz > MAX_LZ {
        return None;
    }
    // `|g| ≤ 1/256` leaves at least seven leading zeros, so every shift below
    // is in range; `|g| = G·2^(−128−lz)` for the top 128 bits `G`.
    let high = (g2 << lz) | (g1 >> (64 - lz));
    let low = (g1 << lz) | (g0 >> (64 - lz));
    let g = (u128::from(high) << 64) | u128::from(low);
    let (high, low) = wmul(g, PIO2_128);
    let lzt = high.leading_zeros();

    Some(Residual {
        n,
        negative,
        t1: top_256([low, high], lzt),
        et: 1 - (lz + lzt) as i32,
    })
}

/// `(u, v, u1)`: `θ²` rounded into a 2^-128 word, `θ⁴` likewise, and the
/// unshifted `θ² = u1·2^(2·et−128)` the cosine's correction rides on.
#[inline]
const fn squares(t1: u128, et: i32) -> (u128, u128, u128) {
    let u1 = mhi_approx(t1, t1);
    let u = shr_round(u1, (-2 * et) as u32);

    (u, mhi_approx(u, u), u1)
}

/// `sin θ` in `θ`'s own floating form: `t1·(1 − u·(A − u·B))` with the even
/// and odd halves of `Σ (−1)^k u^k/(2k+3)!` as two short chains in `v`.  Six
/// terms: the seventh is below 2^-143 for `u < 2^-14.7`.
#[inline]
fn sin_frac(t1: u128, u: u128, v: u128) -> u128 {
    let s = |k: usize| SIN_COEF[k][2];
    let a = s(0) + mhi_approx(v, s(2) + mhi_approx(v, s(4)));
    let b = s(1) + mhi_approx(v, s(3) + mhi_approx(v, s(5)));

    t1 - mhi_approx(mhi_approx(t1, u), a - mhi_approx(u, b))
}

/// `1 − cos θ` as the floating fraction `c1·2^(2·et−128)`: `θ²·(½ − u·W)`
/// with `W` the even and odd halves of `Σ (−1)^k u^k/(2k+4)!` in `v`.  Seven
/// terms: the eighth is below 2^-131 of the correction.
#[inline]
fn cos_corr(u1: u128, u: u128, v: u128) -> u128 {
    let c = |k: usize| COS_COEF[k][2];
    let e = c(1) + mhi_approx(v, c(3) + mhi_approx(v, c(5)));
    let o = c(2) + mhi_approx(v, c(4) + mhi_approx(v, c(6)));

    mhi_approx(u1, c(0) - mhi_approx(u, e - mhi_approx(u, o)))
}

/// The fast leg: the magnitude as a floating 128-bit fraction `frac·2^(e2−128)`
/// with the sign it contributes, or `None` when the reduction cannot vouch for
/// it.  Shared with [`ziv_soundness`].
#[inline]
fn fast(m: u128, e: i32, cosine: bool) -> Option<(u128, i32, u128)> {
    if e < DIRECT {
        let t1 = m << 15;
        let et = e + 1;
        let (u, v, u1) = squares(t1, et);

        return Some(if cosine {
            let c = place_256(cos_corr(u1, u, v), (2 * et + 128) as u32);
            (sub_256([u128::MAX; 2], c)[1], 0, 0)
        } else {
            // `sin θ < θ` drops a leading bit when `t1` starts a binade.
            let s1 = sin_frac(t1, u, v);
            let lz = s1.leading_zeros();
            (s1 << lz, et - lz as i32, 0)
        });
    }
    let r = reduce(m, e)?;
    let (u, v, u1) = squares(r.t1, r.et);
    let s1 = sin_frac(r.t1, u, v);
    // The cosine is the sine a quadrant on; the top bit of the quadrant is
    // the sign, the low bit which of `sin A` and `cos A` is wanted.
    let k = (r.n >> 7) + usize::from(cosine);
    let want_cos = k & 1 != 0;
    let flip = if k & 2 == 0 { 0 } else { SIGN_MASK };
    let j = r.n & 127;

    if j == 0 && !want_cos {
        let lz = s1.leading_zeros();
        let flip = flip ^ if r.negative { SIGN_MASK } else { 0 };
        return Some((s1 << lz, r.et - lz as i32, flip));
    }
    let c1 = cos_corr(u1, u, v);
    let first = &SINCOS[j][usize::from(want_cos)];
    let second = &SINCOS[j][usize::from(!want_cos)];
    let base = [first[1], first[2]];
    let c = place_256(mhi_approx(first[2], c1), (2 * r.et + 128) as u32);
    let s = place_256(mhi_approx(second[2], s1), (r.et + 128) as u32);
    let f = add_signed_256(sub_256(base, c), s, r.negative != want_cos);
    let lz = f[1].leading_zeros();

    Some((top_256(f, lz), -(lz as i32), flip))
}

/// [`reduce`] on nine limbs of 2/π and a 448-bit fraction, the residual as a
/// normalized 384-bit fraction `t·2^(et−384)`.
fn reduce_wide(m: u128, e: i32) -> (usize, bool, [u128; 3], i32) {
    let (window, shift) = window::<9>(e);
    let p: [u64; 12] = product(m, window);
    let base = (126 - shift) as u32;
    debug_assert!((63..=248).contains(&base));
    let i = (base / 64) as usize & 3;
    let bits = base % 64;
    let mut f = [0; 7];

    for (k, limb) in f.iter_mut().enumerate() {
        *limb = funnel_down(p[i + k], p[i + k + 1], bits);
    }
    let quadrant = funnel_down(p[i + 7], p[i + 8], bits) & 3;
    let top = (u128::from(quadrant) << 64) | u128::from(f[6]);
    let n = ((top + (1 << 56)) >> 57) as usize & 511;
    f[6] = f[6].wrapping_sub((n as u64) << 57);
    let negative = (f[6] as i64) < 0;
    let mask = 0u64.wrapping_sub(u64::from(negative));
    let mut carry = negative;

    for limb in &mut f {
        let (v, c) = (*limb ^ mask).overflowing_add(u64::from(carry));
        *limb = v;
        carry = c;
    }
    let lz = f
        .iter()
        .rev()
        .position(|&limb| limb != 0)
        .map_or(448, |k| 64 * k as u32 + f[6 - k].leading_zeros());
    // No binary128 comes within 2^-124 of a multiple of π/2, so the residual
    // keeps well over 300 of its 448 bits.
    debug_assert!(lz < 140);
    let f = shl_limbs(f, lz);
    let g = [
        u128::from(f[1]) | u128::from(f[2]) << 64,
        u128::from(f[3]) | u128::from(f[4]) << 64,
        u128::from(f[5]) | u128::from(f[6]) << 64,
    ];
    let t = mul_hi_384(g, PIO2_384);
    let lzt = t[2].leading_zeros();

    (n, negative, shl_384(t, lzt), 1 - (lz + lzt) as i32)
}

/// `x << shift` on little-endian 64-bit limbs, discarding what leaves them.
fn shl_limbs<const N: usize>(x: [u64; N], shift: u32) -> [u64; N] {
    let word = (shift / 64) as usize;
    let bits = shift % 64;
    let mut result = [0; N];

    for i in word..N {
        let carry = if bits == 0 || i == word {
            0
        } else {
            x[i - word - 1] >> (64 - bits)
        };
        result[i] = (x[i - word] << bits) | carry;
    }
    result
}

/// `Σ_{k≥0} (−1)^k COEF[k]·u^k` at 384 bits by Horner: every step stays
/// positive because `u·COEF[k+1] < COEF[k]`.
fn alternating(u: [u128; 3], coef: &[[u128; 3]]) -> [u128; 3] {
    let mut q = coef[coef.len() - 1];

    for c in coef[..coef.len() - 1].iter().rev() {
        q = sub_384(*c, mul_hi_384(u, q));
    }
    q
}

/// The 384-bit leg: everything the fast leg would not decide.
#[cold]
#[inline(never)]
fn accurate(m: u128, e: i32, cosine: bool, sign: u128) -> f128 {
    let (n, negative, t, et) = if e < DIRECT {
        (0, false, [0, 0, m << 15], e + 1)
    } else {
        reduce_wide(m, e)
    };
    let u = shr_384_sat(mul_hi_384(t, t), (-2 * et) as u32);
    let k = (n >> 7) + usize::from(cosine);
    let want_cos = k & 1 != 0;
    let sign = sign ^ if k & 2 == 0 { 0 } else { SIGN_MASK };
    let j = n & 127;

    // `sin θ = θ·(1 − u·Q)` keeps `θ`'s floating form, at most one bit short.
    let s = sub_384(t, mul_hi_384(t, mul_hi_384(u, alternating(u, &SIN_COEF))));
    let lzs = leading_zeros_384(s);
    let s = shl_384(s, lzs);
    let es = et - lzs as i32;

    if j == 0 && !want_cos {
        return round_384(s, es, sign ^ if negative { SIGN_MASK } else { 0 });
    }
    // `1 − cos θ = u·Q`, so `T·cos θ = T − T·(u·Q)` never overflows the frame.
    let c = mul_hi_384(u, alternating(u, &COS_COEF));
    let first = SINCOS[j][usize::from(want_cos)];
    let second = SINCOS[j][usize::from(!want_cos)];
    let base = sub_384(first, mul_hi_384(first, c));
    let term = shr_384_sat(mul_hi_384(second, s), (-es) as u32);
    let f = if negative != want_cos {
        sub_384(base, term)
    } else {
        add_384(base, term)
    };
    let lz = leading_zeros_384(f);

    round_384(shl_384(f, lz), -(lz as i32), sign)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f128::consts;

    #[test]
    fn specials() {
        assert_eq!(sinq(0.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(sinq(-0.0).to_bits(), (-0.0_f128).to_bits());
        assert_eq!(cosq(0.0).to_bits(), 1.0_f128.to_bits());
        assert_eq!(cosq(-0.0).to_bits(), 1.0_f128.to_bits());
        assert!(sinq(f128::INFINITY).is_nan());
        assert!(sinq(f128::NEG_INFINITY).is_nan());
        assert!(cosq(f128::INFINITY).is_nan());
        assert!(sinq(f128::NAN).is_nan());
        assert!(cosq(f128::NAN).is_nan());
        // The tiny band: the sine is the argument, the cosine one.
        for x in [
            f128::from_bits(1),
            f128::MIN_POSITIVE,
            crate::f128_::exp2i(-58),
            f128::from_bits(TINY - 1),
        ] {
            assert_eq!(sinq(x).to_bits(), x.to_bits());
            assert_eq!(sinq(-x).to_bits(), (-x).to_bits());
            assert_eq!(cosq(x).to_bits(), 1.0_f128.to_bits());
        }
    }

    #[test]
    fn symmetry() {
        for x in [0.5_f128, 1.0, 3.0, 100.0, 1e30, crate::f128_::exp2i(-20)] {
            assert_eq!(sinq(-x).to_bits(), (-sinq(x)).to_bits());
            assert_eq!(cosq(-x).to_bits(), cosq(x).to_bits());
        }
    }

    #[test]
    fn known_values() {
        // mpmath golden values at 113-bit round-to-nearest.
        let check = |x: f128, sin: u128, cos: u128| {
            assert_eq!(sinq(x).to_bits(), sin, "sin {x:?}");
            assert_eq!(cosq(x).to_bits(), cos, "cos {x:?}");
        };
        check(
            1.0,
            0x3ffe_aed5_48f0_90ce_e041_8dd3_d213_8a1e,
            0x3ffe_14a2_80fb_5068_b923_848c_db2e_d0e3,
        );
        check(
            consts::FRAC_PI_2,
            1.0_f128.to_bits(),
            0x3f8c_cd12_9024_e088_a67c_c740_20bb_ea64,
        );
        check(
            consts::PI,
            0x3f8d_cd12_9024_e088_a67c_c740_20bb_ea64,
            (-1.0_f128).to_bits(),
        );
        check(
            crate::f128_::exp2i(-10),
            0x3ff4_ffff_faaa_aaae_eeee_ed4e_d4ed_ab4c,
            0x3ffe_ffff_f000_0015_5555_49f4_9f4d_34d3,
        );
        check(
            crate::f128_::exp2i(100),
            0xbffe_be8e_d97a_c1f5_8bea_99cc_1e48_9ea6,
            0x3ffd_f4eb_3ff6_6e36_cd2d_c761_b1ff_bcc6,
        );
        check(
            f128::MAX,
            0x3ffe_e761_623d_b0b6_ffc8_7a22_04a2_b017,
            0xbffd_39b7_257e_d4a6_f0d9_7d1c_b93e_f07e,
        );
    }
}

/// MPFR certification that `atan2q`'s [`ZIV_GATE`] covers the fast leg's
/// true error with the 2× margin the project requires.  Run with
/// `CC=clang cargo +nightly test --release --features "f128 mpfr"`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::super::MANTISSA_MASK;
    use super::super::atan2::ZIV_GATE;
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

    /// `round(k·π/2) + 2^-t` for `k < 2^30` and `t ∈ [7, 60]`: a residual of
    /// about `2^-t`, spanning the fast leg's whole admissible range down to
    /// its hand-over.
    fn near_multiple(i: u64) -> f128 {
        let k = mix(i) % (1 << 30) + 1;
        let t = 7 + (mix(!i) % 54) as i32;
        let x = Float::with_val(PRECISION, Constant::Pi) * k / 2u32;
        let x = x.to_f128_round(rug::float::Round::Nearest);

        x + crate::f128_::exp2i(-i64::from(t))
    }

    /// A binade edge: a mantissa within 2^-100 of a power of two from either
    /// side, where the series' leading bit moves.
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

    /// `|frame − |f(x)|| / ZIV_GATE` in gate units of 2^(e2−128), or `None`
    /// where the fast leg hands over; the sign is checked on the way.
    fn slip(x: f128, cosine: bool) -> Option<f64> {
        let (m, e) = split(x.to_bits() & !SIGN_MASK);
        let (frac, e2, flip) = fast(m, e, cosine)?;
        let value = Float::with_val(PRECISION, x);
        let truth = if cosine { value.cos() } else { value.sin() };
        assert_eq!(
            flip != 0,
            truth.is_sign_negative(),
            "{} sign at {x:?}",
            if cosine { "cosq" } else { "sinq" }
        );
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
        let mut worst_at = (0.0_f128, false);
        let mut handed_over = 0;

        for i in 0..SAMPLES {
            let x = draw(i);

            for cosine in [false, true] {
                let Some(ratio) = slip(x, cosine) else {
                    handed_over += 1;
                    continue;
                };
                if ratio > worst {
                    worst = ratio;
                    worst_at = (x, cosine);
                }
            }
        }
        println!(
            "sinq/cosq fast leg: worst |err|/gate = {worst:.4} at {worst_at:?} ({handed_over} handed over)"
        );
        assert!(
            worst < 0.5,
            "sinq/cosq gate covers only {:.2}× the slip at {worst_at:?}",
            1.0 / worst
        );
    }
}
