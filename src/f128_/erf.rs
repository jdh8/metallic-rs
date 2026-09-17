//! Binary128 error functions, with original Taylor expansions throughout.
//!
//! Below 1/2, erf(x)/x is an alternating factorial series, multiplied by the
//! exact input significand last. Above it, erfc(x)=exp(-x²)*erfcx(x): the square
//! is exact, and sixteen centers per binade leave |(x-c)/c| <= 1/33. Every
//! magnitude in the Taylor series erfcx(c*(1+t))/erfcx(c) is at most one.
//! Its first 28 terms leave less than 2^-140; 78 terms leave below 2^-393.
//!
//! The fallback uses 384-bit arithmetic, including the unrounded exponential
//! of the exact square. The 380-bit log2(e) constant loses at most 14 bits to
//! x²; the resulting relative precision policy is 2^-360. This is a domain
//! precision policy, not an exhaustive proof for all binary128 inputs.

use super::erf_tables::{ERF, RATIO, VALUES};
use super::exp;
use super::exp_tables::LOG2E;
use super::hyp::exp2_384;
use super::tan::{quotient, quotient_384};
use super::uint::{
    add_384, leading_zeros_384, mhi, mul_hi_64, mul_hi_384, neg_384, shl_384, shr_384_sat, shr_sat,
    sub_384, wmul, wmul_128x256, wmul_128x384,
};
use super::{EXP_MASK, QUIET_BIT, SIGN_MASK, split};

const ONE: u128 = 1 << 127;
// erfc(9) < exp(-81)/(9*sqrt(pi)) < 2^-120, below both half ulps.
const SATURATE: u128 = 9.0_f128.to_bits();
const UNDERFLOW: u128 = 108.0_f128.to_bits();

/// The error function, (2/√π) ∫₀ˣ exp(−t²) dt.
#[must_use]
pub fn erfq(x: f128) -> f128 {
    error_function::<false>(x)
}

/// The complementary error function, 1 − erf(x).
#[must_use]
pub fn erfcq(x: f128) -> f128 {
    error_function::<true>(x)
}

#[inline]
fn error_function<const COMPLEMENT: bool>(x: f128) -> f128 {
    let bits = x.to_bits();
    let a = bits & !SIGN_MASK;
    let negative = bits & SIGN_MASK != 0;
    if a > EXP_MASK {
        return f128::from_bits(bits | QUIET_BIT);
    }
    if a == 0 {
        return if COMPLEMENT { 1.0 } else { x };
    }
    if a >= SATURATE && (!COMPLEMENT || negative) {
        return if COMPLEMENT {
            2.0
        } else if negative {
            -1.0
        } else {
            1.0
        };
    }
    if a >= UNDERFLOW {
        return 0.0;
    }
    let (m, e) = split(a);
    let (n, r, gate) = fast::<COMPLEMENT>(m, e, negative);
    let y = if exp::undecided(n, r, gate) {
        let (n, r) = wide::<COMPLEMENT>(m, e, negative);
        exp::round(n, r[2], r[1] | u128::from(r[0] != 0))
    } else {
        exp::round(n, r, 0)
    };
    if !COMPLEMENT && negative {
        f128::from_bits(y.to_bits() | SIGN_MASK)
    } else {
        y
    }
}

#[inline]
fn normalized(n: i32, r: u128, gate: u128) -> (i32, u128, u128) {
    let lz = r.leading_zeros();
    (n - lz as i32, r << lz, gate << lz)
}

#[inline]
fn product(a: u128, b: u128, n: i32) -> (i32, u128) {
    let (hi, lo) = wmul(a, b);
    if hi & ONE != 0 {
        (n + 1, hi)
    } else {
        let r = hi << 1 | lo >> 127;
        let lz = r.leading_zeros();
        (n - lz as i32, r << lz)
    }
}

/// Exact input times a Taylor ratio, preserving the irrational tiny slope.
/// Squaring, coefficient cuts, the contracted Horner chain, and the final
/// product together contribute under eight normalized units; the gate is 32.
#[inline]
fn small(m: u128, e: i32) -> (i32, u128, u128) {
    let a = m << 15;
    let u = mhi(a, a).checked_shr((-2 * e - 2) as u32).unwrap_or(0);
    let terms = if e < -64 {
        1
    } else if e < -32 {
        3
    } else if e < -16 {
        5
    } else if e < -8 {
        9
    } else if e < -4 {
        15
    } else if e < -2 {
        21
    } else {
        26
    };
    let mut p = ERF[terms - 1][2];
    for c in ERF[..terms - 1].iter().rev() {
        p = c[2] - mhi(u, p);
    }
    let (n, r) = product(a, p, e);
    (n, r, 32)
}

/// The exact bin center and residual in the input's significand frame.
#[inline]
fn center(m: u128, e: i32) -> (usize, u128, u128, bool) {
    let j = ((m >> 108) - 16) as usize;
    let c = (33 + 2 * j as u128) << 107;
    ((e + 1) as usize * 16 + j, c, m.abs_diff(c), m >= c)
}

/// |x-c|/c at 2^128, with an absolute error below 2^-128.
#[inline]
fn residual(c: u128, d: u128) -> u128 {
    if d == 0 {
        return 0;
    }
    let lc = c.leading_zeros();
    let ld = d.leading_zeros();
    let (q, eq) = quotient(d << ld, c << lc);
    q >> (ld as i32 - lc as i32 - eq)
}

/// Fraction and floor of -x²*log2(e), from the full exact 226-bit square.
#[inline]
fn square_frame(m: u128, e: i32) -> (i32, u128) {
    let (hi, lo) = wmul(m, m);
    let a = wmul_128x256(lo, LOG2E.head);
    let b = wmul_128x256(hi, LOG2E.head);
    let (p1, c1) = a[1].overflowing_add(b[0]);
    let (p2, c2) = a[2].overflowing_add(b[1]);
    let (p2, c3) = p2.overflowing_add(u128::from(c1));
    let p = [a[0], p1, p2, b[2] + u128::from(c2) + u128::from(c3)];
    let y = shr_sat(p, (348 - 2 * e) as u32);
    // The irrational reduction has a nonzero tail, so complementing (rather
    // than negating) floors its negative even when the stored fraction is zero.
    (-1 - y[1] as i32, !y[0])
}

/// The ratio's Taylor remainder is below 2^-140. Coefficient cuts, the
/// quotient, and the contracted Horner chain contribute under eight units;
/// normalization and the exponential product keep the total under 32.
/// A 128-unit gate includes a fourfold allowance before measurement.
#[inline]
fn tail(m: u128, e: i32) -> (i32, u128, u128) {
    let (i, c, d, subtract) = center(m, e);
    let t = residual(c, d);
    let row = &RATIO[i];
    // Above degree fourteen, 64 bits of coefficient and residual leave
    // under 2^-138 in the result because |t| < 2^-5.
    let narrow = (t >> 64) as u64;
    let mut short = (row[27][2] >> 64) as u64;
    for coefficient in row[14..27].iter().rev() {
        let q = mul_hi_64(narrow, short);
        let c = (coefficient[2] >> 64) as u64;
        short = if subtract { c - q } else { c + q };
    }
    let mut p = u128::from(short) << 64;
    for coefficient in row[..14].iter().rev() {
        let q = mhi(t, p);
        p = if subtract {
            coefficient[2] - q
        } else {
            coefficient[2] + q
        };
    }
    let (nc, c) = product(VALUES[i].1[2], p, VALUES[i].0);
    let (n, f) = square_frame(m, e);
    let (ne, r) = exp::fast(n, f);
    let (n, r) = product(c, r, nc + ne);
    (n, r, 128)
}

/// Form `base +/- value` in a common frame without intermediate rounding.
#[inline]
fn complement(n: i32, r: u128, gate: u128, base: u128, add: bool) -> (i32, u128, u128) {
    let shift = (1 - n) as u32;
    let delta = r.checked_shr(shift).unwrap_or(0);
    let z = if add { base + delta } else { base - delta };
    let g = gate.checked_shr(shift).unwrap_or(0) + 4;
    normalized(1, z, g)
}

/// Exactly the raw significand and gate measured by `ziv_soundness`.
fn fast<const COMPLEMENT: bool>(m: u128, e: i32, negative: bool) -> (i32, u128, u128) {
    if e < -1 {
        let (n, r, gate) = small(m, e);
        if COMPLEMENT {
            complement(n, r, gate, 1 << 126, negative)
        } else {
            (n, r, gate)
        }
    } else {
        let (n, r, gate) = tail(m, e);
        if !COMPLEMENT {
            complement(n, r, gate, 1 << 126, false)
        } else if negative {
            complement(n, r, gate, 1 << 127, false)
        } else {
            (n, r, gate)
        }
    }
}

fn normalized_384(n: i32, r: [u128; 3]) -> (i32, [u128; 3]) {
    let lz = leading_zeros_384(r);
    (n - lz as i32, shl_384(r, lz))
}

fn product_384(a: [u128; 3], b: [u128; 3], n: i32) -> (i32, [u128; 3]) {
    let p = mul_hi_384(a, b);
    normalized_384(n + 1, p)
}

fn small_384(m: u128, e: i32) -> (i32, [u128; 3]) {
    let (hi, lo) = wmul(m, m);
    let shift = 2 * e + 160;
    let square = [lo, hi, 0];
    let u = if shift >= 0 {
        shl_384(square, shift as u32)
    } else {
        shr_384_sat(square, (-shift) as u32)
    };
    let mut p = ERF[63];
    for c in ERF[..63].iter().rev() {
        p = sub_384(*c, mul_hi_384(u, p));
    }
    let a = [0, 0, m << 15];
    product_384(a, p, e)
}

fn residual_384(c: u128, d: u128) -> [u128; 3] {
    if d == 0 {
        return [0; 3];
    }
    let lc = c.leading_zeros();
    let ld = d.leading_zeros();
    let (q, eq) = quotient_384([0, 0, d << ld], [0, 0, c << lc]);
    shr_384_sat(q, (ld as i32 - lc as i32 - eq) as u32)
}

fn square_frame_384(m: u128, e: i32) -> (i32, [u128; 3]) {
    let (hi, lo) = wmul(m, m);
    let l = [LOG2E.tail, LOG2E.head[0], LOG2E.head[1]];
    let a = wmul_128x384(lo, l);
    let b = wmul_128x384(hi, l);
    let mut p = [a[0], 0, 0, 0, 0];
    let mut carry = false;
    for j in 1..4 {
        let (s, c1) = a[j].overflowing_add(b[j - 1]);
        let (s, c2) = s.overflowing_add(u128::from(carry));
        p[j] = s;
        carry = c1 || c2;
    }
    p[4] = b[3] + u128::from(carry);
    let y = shr_sat(p, (220 - 2 * e) as u32);
    // Less than 2^-360 relative error including the omitted reduction tail.
    (-1 - y[3] as i32, neg_384([y[0], y[1], y[2]]))
}

fn tail_384(m: u128, e: i32) -> (i32, [u128; 3]) {
    let (i, c, d, subtract) = center(m, e);
    let t = residual_384(c, d);
    let row = &RATIO[i];
    let mut p = row[77];
    for coefficient in row[..77].iter().rev() {
        let q = mul_hi_384(t, p);
        p = if subtract {
            sub_384(*coefficient, q)
        } else {
            add_384(*coefficient, q)
        };
    }
    let (nc, c) = product_384(VALUES[i].1, p, VALUES[i].0);
    let (n, f) = square_frame_384(m, e);
    let (ne, r) = exp2_384(n, f);
    product_384(c, r, nc + ne)
}

fn complement_384(n: i32, r: [u128; 3], base: u128, add: bool) -> (i32, [u128; 3]) {
    let delta = shr_384_sat(r, (1 - n) as u32);
    let base = [0, 0, base];
    normalized_384(
        1,
        if add {
            add_384(base, delta)
        } else {
            sub_384(base, delta)
        },
    )
}

fn wide<const COMPLEMENT: bool>(m: u128, e: i32, negative: bool) -> (i32, [u128; 3]) {
    if e < -1 {
        let (n, r) = small_384(m, e);
        if COMPLEMENT {
            complement_384(n, r, 1 << 126, negative)
        } else {
            (n, r)
        }
    } else {
        let (n, r) = tail_384(m, e);
        if !COMPLEMENT {
            complement_384(n, r, 1 << 126, false)
        } else if negative {
            complement_384(n, r, 1 << 127, false)
        } else {
            (n, r)
        }
    }
}

#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::super::BIAS;
    use super::*;
    use rug::{Float, ops::Pow};

    fn mix(mut i: u64) -> u64 {
        i = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        i = (i ^ (i >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        i = (i ^ (i >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        i ^ (i >> 31)
    }

    fn certify<const COMPLEMENT: bool>() {
        let mut worst = [0.0_f64; 3];
        let mut at = [0.0_f128; 3];
        let mut wide_worst = 0.0_f64;
        for i in 0..1_000_000_u64 {
            let b = u128::from(mix(i)) | u128::from(mix(i ^ 0xABCD)) << 64;
            let e = match i % 4 {
                0 => -16494 + (i / 4 % 16499) as i32,
                1 => -1 + (i / 4 % 8) as i32,
                2 => [-65, -64, -18, -17, -16, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6]
                    [(i / 4 % 15) as usize],
                _ => -114 + (i / 4 % 121) as i32,
            };
            let xb = if e < -16382 {
                (b & ((1 << 112) - 1)).max(1) >> (-16382 - e).min(111)
            } else {
                ((BIAS + e) as u128) << 112 | (b & ((1 << 112) - 1))
            };
            if xb == 0 || xb >= UNDERFLOW {
                continue;
            }
            let negative = i & 16 != 0;
            if xb >= SATURATE && (!COMPLEMENT || negative) {
                continue;
            }
            let x = f128::from_bits(xb | if negative { SIGN_MASK } else { 0 });
            let (m, e) = split(xb);
            let (n, r, gate) = fast::<COMPLEMENT>(m, e, negative);
            let z = Float::with_val(448, x);
            let truth = if COMPLEMENT { z.erfc() } else { z.erf().abs() };
            let unit = Float::with_val(448, 2).pow(n - 127);
            let ratio = (Float::with_val(448, &truth / unit) - r).abs().to_f64() / gate as f64;
            let band = if e < -1 {
                0
            } else if negative {
                1
            } else {
                2
            };
            if ratio > worst[band] {
                worst[band] = ratio;
                at[band] = x;
            }
            if i % 127 == 0 {
                let (n, r) = wide::<COMPLEMENT>(m, e, negative);
                let raw = Float::with_val(448, r[2]) * Float::with_val(448, 2).pow(256)
                    + Float::with_val(448, r[1]) * Float::with_val(448, 2).pow(128)
                    + r[0];
                let actual: Float = raw * Float::with_val(448, 2).pow(n - 383);
                let relative: Float =
                    ((actual - &truth) / &truth).abs() * Float::with_val(448, 2).pow(360);
                wide_worst = wide_worst.max(relative.to_f64());
            }
        }
        println!(
            "erf complement={COMPLEMENT}: worst |err|/gate {worst:?} at {at:?}; wide |relative err|/2^-360={wide_worst:e}"
        );
        assert!(worst.into_iter().all(|x| x < 0.5));
        assert!(wide_worst < 0.5);
    }

    #[test]
    fn erfq_fast_legs_are_sound() {
        certify::<false>();
    }
    #[test]
    fn erfcq_fast_legs_are_sound() {
        certify::<true>();
    }

    #[test]
    fn forced_accurate_matches_mpfr() {
        for i in 0..20_000_u64 {
            let b = u128::from(mix(i)) | u128::from(mix(i ^ 0xDECA)) << 64;
            let e = -120 + (i % 127) as i32;
            let xb = ((BIAS + e) as u128) << 112 | (b & ((1 << 112) - 1));
            if xb >= UNDERFLOW {
                continue;
            }
            for negative in [false, true] {
                let x = f128::from_bits(xb | if negative { SIGN_MASK } else { 0 });
                let (m, e) = split(xb);
                for complementary in [false, true] {
                    if xb >= SATURATE && (!complementary || negative) {
                        continue;
                    }
                    let (n, r) = if complementary {
                        wide::<true>(m, e, negative)
                    } else {
                        wide::<false>(m, e, negative)
                    };
                    let y = exp::round(n, r[2], r[1] | u128::from(r[0] != 0));
                    let truth = if complementary {
                        Float::with_val(113, x).erfc()
                    } else {
                        Float::with_val(113, x).erf().abs()
                    };
                    // This band has normal results; subnormal results belong
                    // to the public MPFR oracle's ternary-aware subnormalizer.
                    if y >= f128::MIN_POSITIVE {
                        assert_eq!(
                            y.to_bits(),
                            truth.to_f128().to_bits(),
                            "x={x:?}, erfc={complementary}"
                        );
                    }
                }
            }
        }
    }
}
