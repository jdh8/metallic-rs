//! Binary128 inverse hyperbolics, with unrounded integer roots and logarithms.
//!
//! asinh(x) = log(x + sqrt(x²+1)) for x>0; acosh(x) replaces +1 by -1.
//! Small asinh arguments use its alternating Taylor ratio. Near one, acosh
//! uses 2*asinh(sqrt((x-1)/2)), keeping the exact subtraction in integers.
//! No binary128 operation is needed after decoding, and no intermediate root
//! or logarithm rounds to binary128. Large inputs use log(2x) only after the
//! omitted correction fits the active leg's bound.
//!
//! The accurate leg has a relative precision policy of 2^-320: 384-bit roots
//! and series, then the existing logarithm tables at 2^-342 with a 384-bit
//! residual series. The smallest logarithmic result exceeds 2^-5. Reduction
//! cuts at 2^-333 and the table's exponent term slips by at most 2^-328 at
//! the largest input. This is a precision policy, not an exhaustive proof;
//! the domain heuristic predicts fewer than 2^-78 unresolved inputs.
//! Lindemann-Weierstrass excludes nonzero algebraic results for these
//! logarithms of algebraic numbers, so the only exact cases are asinh(0)
//! and acosh(1); there are no nontrivial dyadic midpoint cases.

use super::atan2::{round_384, round_fast};
use super::invhyp_tables::{ASINH, LOG};
use super::log::{self, Natural};
use super::log_tables::{LN2, LOG0, LOG1, LOG2};
use super::roots::sqrt_wide_seeded;
use super::tan::quotient_384;
use super::uint::{
    add_256, add_384, leading_zeros_256, leading_zeros_384, mhi, mul_hi_384, neg_384, shl_256,
    shl_384, shr_256_sat, shr_384_sat, sub_256, sub_384, wmul, wmul_128x384,
};
use super::{BIAS, EXP_MASK, EXP_SHIFT, QUIET_BIT, SIGN_MASK, split};

const ONE: u128 = (BIAS as u128) << EXP_SHIFT;
const TINY: u128 = ((BIAS - 57) as u128) << EXP_SHIFT;
const DIRECT: i32 = -4;
// Square/ratio truncation, the final product and at most one normalization
// bit cost under eight units, including acosh's truncated root. The omitted
// Taylor tail is below 2^-135 relative. 32 units retain at least 2x margin.
const SERIES_GATE: u128 = 32;
const NEAR_ONE: u128 = ONE + (1 << 105); // 1 + 2^-7

/// The inverse hyperbolic sine, preserving signed zero.
#[must_use]
pub fn asinhq(x: f128) -> f128 {
    let bits = x.to_bits();
    let a = bits & !SIGN_MASK;
    if a >= EXP_MASK {
        return f128::from_bits(bits | if a > EXP_MASK { QUIET_BIT } else { 0 });
    }
    // |asinh(x)-x| < |x|³/6, strictly below half an ulp here.
    if a <= TINY {
        return x;
    }
    inverse::<false>(a, bits & SIGN_MASK)
}

/// The inverse hyperbolic cosine, with domain x ≥ 1 and acosh(1) = +0.
#[must_use]
pub fn acoshq(x: f128) -> f128 {
    let bits = x.to_bits();
    if bits & !SIGN_MASK > EXP_MASK {
        return f128::from_bits(bits | QUIET_BIT);
    }
    if bits < ONE || bits & SIGN_MASK != 0 {
        return f128::NAN;
    }
    if bits == ONE {
        return 0.0;
    }
    if bits == EXP_MASK {
        return x;
    }
    inverse::<true>(bits, 0)
}

#[inline]
fn inverse<const COSH: bool>(bits: u128, sign: u128) -> f128 {
    let (r, e, gate) = fast::<COSH>(bits);
    round_fast(r, e, sign, gate).unwrap_or_else(|| {
        let (r, e) = wide::<COSH>(bits);
        round_384(r, e, sign)
    })
}

#[inline]
fn normalize(r: [u128; 2], e: i32) -> (u128, i32) {
    let lz = leading_zeros_256(r);
    (shl_256(r, lz)[1], e - lz as i32)
}

fn normalize_384(r: [u128; 3], e: i32) -> ([u128; 3], i32) {
    let lz = leading_zeros_384(r);
    (shl_384(r, lz), e - lz as i32)
}

/// sqrt(v/2^256) = s/2^256 * 2^-shift, within 2^-158 relative.
/// One Newton correction against the entire square uses the same seed as
/// the initial root; its ~2^-43 error multiplies a residual below 2^-119.
#[inline]
fn root(v: [u128; 2]) -> ([u128; 2], u32) {
    let parity = leading_zeros_256(v) & !1;
    let vn = shl_256(v, parity);
    let (s0, r) = sqrt_wide_seeded(vn[1]);
    // A root arbitrarily close to one can round its seed to the endpoint.
    let s0 = s0.min((1 << 126) - 1);
    let (qh, ql) = wmul(s0, s0);
    let square = shl_256([ql, qh], 4);
    let downward = (square[1], square[0]) > (vn[1], vn[0]);
    let res = if downward {
        sub_256(square, vn)
    } else {
        sub_256(vn, square)
    };
    debug_assert!(res[1] < 1 << 12);
    let top = res[1] << 116 | res[0] >> 12;
    let (ph, pl) = wmul(top, u128::from(r));
    let corr = [pl >> 51 | ph << 77, ph >> 51];
    let s = if downward {
        sub_256([0, s0 << 2], corr)
    } else {
        let sum = add_256([0, s0 << 2], corr);
        if sum[1] < s0 << 2 {
            [u128::MAX; 2]
        } else {
            sum
        }
    };
    (s, parity / 2)
}

/// asinh(a*2^(e-128)), |x|<2^-4, in the same floating frame.
/// Integer fixed-point polynomials cannot use the floating poly helper.
#[inline]
fn series(a: u128, e: i32) -> (u128, i32) {
    let u = mhi(a, a) >> (-2 * e);
    let terms = if e <= -16 { 4 } else { 16 };
    let mut p = ASINH[terms - 1][2];
    for c in ASINH[..terms - 1].iter().rev() {
        p = c[2] - mhi(u, p);
    }
    let ratio = (1 << 127) - (mhi(u, p) >> 1);
    let (hi, lo) = wmul(a, ratio);
    normalize([lo, hi], e + 1)
}

/// x² +/- 1 at scale 2^(256-2k), where k=max(e+2,1).
/// Below the large-input cutoff the product and +/-1 are both exact.
#[inline]
fn square<const COSH: bool>(m: u128, e: i32) -> ([u128; 2], i32) {
    let k = (e + 2).max(1);
    let (hi, lo) = wmul(m, m);
    let xx = shl_256([lo, hi], (32 + 2 * e - 2 * k) as u32);
    let one = shl_256([1, 0], (256 - 2 * k) as u32);
    (
        if COSH {
            sub_256(xx, one)
        } else {
            add_256(xx, one)
        },
        k,
    )
}

/// Normalized x+sqrt(x² +/- 1), with no binary128 rounding.
#[inline]
fn argument<const COSH: bool>(m: u128, e: i32) -> ([u128; 2], i32) {
    let (v, k) = square::<COSH>(m, e);
    let (s, shift) = root(v);
    let a = shl_256([m, 0], (143 + e - k) as u32);
    let sum = add_256(a, shr_256_sat(s, shift + 1));
    let lz = leading_zeros_256(sum);
    (shl_256(sum, lz), k - lz as i32)
}

/// Raw fraction, exponent and gate measured by the MPFR certification.
#[inline]
fn fast<const COSH: bool>(bits: u128) -> (u128, i32, u128) {
    let (m, e) = split(bits);
    if !COSH && e < DIRECT {
        let (r, e) = series(m << 15, e + 1);
        return (r, e, SERIES_GATE);
    }
    if COSH && bits < NEAR_ONE {
        // h=(x-1)/2 is exact; acosh(x)=2*asinh(sqrt(h)).
        let (s, shift) = root(shl_256([bits - ONE, 0], 143));
        let (a, e) = normalize(s, -(shift as i32));
        let (r, e) = series(a, e);
        return (r, e + 1, SERIES_GATE);
    }
    let (big, e) = if e >= 80 {
        // |asinh(x)-log(2x)|, |acosh(x)-log(2x)| < 1/(2x²).
        // At x>=2^80 this is <2^-161, below the log gate's 2^-139.
        ([0, m << 15], e + 1)
    } else {
        argument::<COSH>(m, e)
    };
    let (e, j, d) = log::reduce_significand(big, e);
    let s = log::fast::<Natural>(e, j, log::z_fast(d));
    let (r, e) = normalize(s, 42); // log's frame is at 2^-214.
    // The log/root slip is under 0.02 normalized units here; truncating the
    // frame to 128 bits adds under one. Four units retain >2x headroom.
    (r, e, 4)
}

/// A 384-bit root, in the same fractional frame as root(). Two Newton
/// averages from its ~119-bit seed reach the quotient's 2^-377 precision.
fn root_384(v: [u128; 3]) -> ([u128; 3], u32) {
    let parity = leading_zeros_384(v) & !1;
    let vn = shl_384(v, parity);
    let s0 = sqrt_wide_seeded(vn[2]).0.min((1 << 126) - 1);
    let mut s = [0, 0, s0 << 2];
    let lz = leading_zeros_384(vn);
    let numerator = shl_384(vn, lz);
    for _ in 0..2 {
        let (q, eq) = quotient_384(numerator, s);
        let q = shr_384_sat(q, (1 + lz as i32 - eq) as u32);
        let half = shr_384_sat(s, 1);
        let sum = add_384(half, q);
        // Newton can overshoot one when sqrt(v) is closer than the
        // squared seed error. Saturating keeps the next divisor normalized.
        s = if sum[2] < half[2] {
            [u128::MAX; 3]
        } else {
            sum
        };
    }
    (s, parity / 2)
}

fn series_384(a: [u128; 3], e: i32) -> ([u128; 3], i32) {
    let u = shr_384_sat(mul_hi_384(a, a), (-2 * e) as u32);
    let mut p = ASINH[ASINH.len() - 1];
    for c in ASINH[..ASINH.len() - 1].iter().rev() {
        p = sub_384(*c, mul_hi_384(u, p));
    }
    let ratio = sub_384([0, 0, 1 << 127], shr_384_sat(mul_hi_384(u, p), 1));
    normalize_384(mul_hi_384(a, ratio), e + 1)
}

fn argument_384<const COSH: bool>(m: u128, e: i32) -> ([u128; 3], i32) {
    let k = (e + 2).max(1);
    let (hi, lo) = wmul(m, m);
    let xx = shl_384([lo, hi, 0], (160 + 2 * e - 2 * k) as u32);
    let one = if 2 * k <= 384 {
        shl_384([1, 0, 0], (384 - 2 * k) as u32)
    } else {
        [0; 3]
    };
    let v = if COSH {
        sub_384(xx, one)
    } else {
        add_384(xx, one)
    };
    let (s, shift) = root_384(v);
    let a = shl_384([m, 0, 0], (271 + e - k) as u32);
    let sum = add_384(a, shr_384_sat(s, shift + 1));
    let lz = leading_zeros_384(sum);
    (shl_384(sum, lz), k - lz as i32)
}

/// Natural log of a 384-bit significand. Reuse logq's reduction and add-back
/// tables, with a longer residual series to keep the 2^-320 precision policy.
fn logarithm(big: [u128; 3], e: i32) -> ([u128; 3], i32) {
    let j = log::crude_log2(big[2] >> 15);
    let p = wmul_128x384(log::reciprocal(j), big);
    // Product at 2^476; cut to 2^333 and subtract one in two's complement.
    let d = [
        p[1] >> 15 | p[2] << 113,
        p[2] >> 15 | p[3] << 113,
        (p[3] >> 15).wrapping_sub(1 << 77),
    ];
    let negative = d[2] >> 127 != 0;
    let z = shl_384(if negative { neg_384(d) } else { d }, 50);
    let mut q = LOG[19];
    for c in LOG[..19].iter().rev() {
        let t = shl_384(mul_hi_384(z, q), 1);
        q = if negative {
            add_384(*c, t)
        } else {
            sub_384(*c, t)
        };
    }
    let residual = shr_384_sat(mul_hi_384(z, q), 40); // 2^382 -> 2^342
    let p = wmul_128x384(e.unsigned_abs() as u128, LN2);
    let mut sum = [p[0], p[1], p[2]];
    if e < 0 {
        sum = neg_384(sum);
    }
    for t in [
        LOG0[(j >> 12) as usize],
        LOG1[(j >> 6 & 63) as usize],
        LOG2[(j & 63) as usize],
    ] {
        sum = add_384(sum, t);
    }
    sum = if negative {
        sub_384(sum, residual)
    } else {
        add_384(sum, residual)
    };
    normalize_384(sum, 42)
}

#[cold]
#[inline(never)]
fn wide<const COSH: bool>(bits: u128) -> ([u128; 3], i32) {
    let (m, e) = split(bits);
    if !COSH && e < DIRECT {
        return series_384([0, 0, m << 15], e + 1);
    }
    if COSH && bits < NEAR_ONE {
        let (s, shift) = root_384(shl_384([bits - ONE, 0, 0], 271));
        let (a, e) = normalize_384(s, -(shift as i32));
        let (r, e) = series_384(a, e);
        return (r, e + 1);
    }
    let (big, e) = if e >= 192 {
        ([0, 0, m << 15], e + 1)
    } else {
        argument_384::<COSH>(m, e)
    };
    logarithm(big, e)
}

#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::*;
    use rug::{Float, float::Round::Nearest, ops::Pow};

    fn mix(mut i: u64) -> u64 {
        i = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        i = (i ^ (i >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        i = (i ^ (i >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        i ^ (i >> 31)
    }
    fn certify<const COSH: bool>() {
        let mut worst = [0.0_f64; 2];
        let mut at = [0.0_f128; 2];
        let mut wide_worst = [0.0_f64; 2];
        let mut refused = [0_u64; 2];
        for i in 0..1_000_000_u64 {
            let b = u128::from(mix(i)) | u128::from(mix(i ^ 0xCAFE)) << 64;
            let e = match i % 4 {
                0 => (i / 4 % 16384) as i32,
                1 => -57 + (i / 4 % 250) as i32,
                2 => [-57, -56, -17, -16, -5, -4, 0, 1, 79, 80, 191, 192][(i / 4 % 12) as usize],
                _ => -57 + (i / 4 % 58) as i32,
            };
            let bits = if COSH && (e < 0 || i % 8 == 0) {
                ONE + ((b & ((1_u128 << (1 + i % 112)) - 1)).max(1))
            } else {
                ((BIAS + e) as u128) << 112 | b & ((1 << 112) - 1)
            };
            let x = f128::from_bits(bits);
            let (r, e, gate) = fast::<COSH>(bits);
            let z = Float::with_val(448, x);
            let truth = if COSH { z.acosh() } else { z.asinh() };
            let unit = Float::with_val(448, 2).pow(e - 128);
            let error = (Float::with_val(448, &truth / unit) - r).abs().to_f64() / gate as f64;
            let band = usize::from(if COSH {
                bits >= NEAR_ONE
            } else {
                bits >= ((BIAS + DIRECT) as u128) << 112
            });
            if error > worst[band] {
                worst[band] = error;
                at[band] = x;
            }
            let got = round_fast(r, e, 0, gate);
            refused[band] += u64::from(got.is_none());
            if let Some(got) = got {
                assert_eq!(
                    got.to_bits(),
                    truth.to_f128_round(Nearest).to_bits(),
                    "fast x={x:?}"
                );
            }
            // Force the fallback independently of the gate, across all bands.
            if i % 127 == 0 || got.is_none() {
                let (r, e) = wide::<COSH>(bits);
                let raw = Float::with_val(448, r[2]) * Float::with_val(448, 2).pow(256)
                    + Float::with_val(448, r[1]) * Float::with_val(448, 2).pow(128)
                    + r[0];
                let actual: Float = raw * Float::with_val(448, 2).pow(e - 384);
                let ratio: Float =
                    ((actual - &truth) / &truth).abs() * Float::with_val(448, 2).pow(320);
                wide_worst[band] = wide_worst[band].max(ratio.to_f64());
                assert_eq!(
                    round_384(r, e, 0).to_bits(),
                    truth.to_f128_round(Nearest).to_bits(),
                    "wide x={x:?}"
                );
            }
        }
        println!(
            "inverse hyperbolic cosh={COSH}: worst |err|/gate {worst:?} at {at:?}; wide |relative err|/2^-320 {wide_worst:?}; refused {refused:?}/1000000"
        );
        assert!(worst.into_iter().all(|v| v < 0.5));
        assert!(wide_worst.into_iter().all(|v| v < 0.5));
    }
    #[test]
    fn accurate_edges() {
        for e in -57..=16383 {
            let anchor = ((BIAS + e) as u128) << EXP_SHIFT;
            for offset in -2..=2_i128 {
                let bits = anchor.wrapping_add_signed(offset);
                let x = f128::from_bits(bits);
                if bits >= TINY {
                    let (r, e) = wide::<false>(bits);
                    let want = super::super::mpfr::cr_unop(x, |y| y.asinh_round(Nearest));
                    assert_eq!(
                        round_384(r, e, 0).to_bits(),
                        want.to_bits(),
                        "asinh wide x={x:?}"
                    );
                }
                if bits > ONE {
                    let (r, e) = wide::<true>(bits);
                    let want = super::super::mpfr::cr_unop(x, |y| y.acosh_round(Nearest));
                    assert_eq!(
                        round_384(r, e, 0).to_bits(),
                        want.to_bits(),
                        "acosh wide x={x:?}"
                    );
                }
            }
        }
    }
    #[test]
    fn asinhq_fast_legs_are_sound() {
        certify::<false>();
    }
    #[test]
    fn acoshq_fast_legs_are_sound() {
        certify::<true>();
    }
}
