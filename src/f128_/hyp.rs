//! Binary128 hyperbolics: direct Taylor ratios near zero, unrounded
//! exponentials elsewhere. All arithmetic after decoding is integer arithmetic.
//!
//! Below 2^-4, sinh(x)/x and cosh(x) have positive factorial series; tanh(x)/x
//! has an alternating Bernoulli series. The input significand is multiplied
//! last, so the tiny correction never suffers subtraction of exponentials.
//! Above it, (exp(x) +/- exp(-x))/2 shares expq's 128-bit engine. The two
//! halves are aligned before adding, keeping both the sum and difference in
//! range even when exp(x) itself would overflow binary128. Tanh uses the
//! shared Newton quotient on those same sum/difference significands.
//!
//! The fallback lifts the same table reduction to 384 bits. Its reduction
//! constant is expq's existing 380-bit log2(e); |x| < 2^14 makes its absolute
//! error below 2^-366. Tables and products add under 2^-375 relative error,
//! and the at-most-five-bit cancellation leaves over 360 relative bits.
//! This is a precision policy (Ziv's domain heuristic < 2^-118 expected
//! misses), not an exhaustive binary128 proof. Nonzero dyadic x cannot give
//! a rational sinh/cosh/tanh value: that would make exp(x) algebraic.

use super::exp;
use super::exp_tables::LOG2E;
use super::hyp_tables::{COSH, EXP, SINH, T0, T1, T2, TANH};
use super::tan::{quotient, quotient_384};
use super::uint::{
    add_384, leading_zeros_384, mhi, mul_hi_384, neg_384, shl_384, shr_384_sat, shr_sat, sub_384,
    wmul, wmul_128x384,
};
use super::{BIAS, EXP_MASK, EXP_SHIFT, QUIET_BIT, SIGN_MASK, split};

const SINE: u8 = 0;
const COSINE: u8 = 1;
const TANGENT: u8 = 2;
const DIRECT: i32 = -4;
const TINY: u128 = ((BIAS - 57) as u128) << EXP_SHIFT;
const LIMIT: u128 = ((BIAS + 14) as u128) << EXP_SHIFT;
const TANH_LIMIT: u128 = ((BIAS + 6) as u128) << EXP_SHIFT;
const ONE: u128 = 1 << 127;

/// The hyperbolic sine, (e<sup>x</sup> − e<sup>−x</sup>)/2.
#[must_use]
pub fn sinhq(x: f128) -> f128 {
    hyperbolic::<SINE>(x)
}

/// The hyperbolic cosine, (e<sup>x</sup> + e<sup>−x</sup>)/2.
#[must_use]
pub fn coshq(x: f128) -> f128 {
    hyperbolic::<COSINE>(x)
}

/// The hyperbolic tangent, sinh(x)/cosh(x).
#[must_use]
pub fn tanhq(x: f128) -> f128 {
    hyperbolic::<TANGENT>(x)
}

#[inline]
fn hyperbolic<const KIND: u8>(x: f128) -> f128 {
    let bits = x.to_bits();
    let a = bits & !SIGN_MASK;
    let sign = if KIND == COSINE { 0 } else { bits & SIGN_MASK };
    let limit = if KIND == TANGENT { TANH_LIMIT } else { LIMIT };
    if a >= limit {
        if a > EXP_MASK {
            return f128::from_bits(bits | QUIET_BIT);
        }
        // 1-tanh(64) < 2*exp(-128) < 2^-183, versus half-ulp 2^-114.
        let y = if KIND == TANGENT {
            1.0_f128
        } else {
            f128::INFINITY
        };
        return f128::from_bits(y.to_bits() | sign);
    }
    if a <= TINY {
        // |sinh(x)/x-1| < x²/5 and |tanh(x)/x-1| < x²/3;
        // cosh(x)-1 < x², all strictly inside their half-ulp at 2^-57.
        return if KIND == COSINE { 1.0 } else { x };
    }
    let (m, e) = split(a);
    let (n, r, gate) = fast::<KIND>(m, e);
    let y = if exp::undecided(n, r, gate) {
        accurate::<KIND>(m, e)
    } else {
        exp::round(n, r, 0)
    };
    f128::from_bits(y.to_bits() | sign)
}

/// Coefficients are exact Taylor constants generated in hyp_tables.rs.
#[inline]
fn coefficients<const KIND: u8>() -> &'static [[u128; 3]] {
    match KIND {
        SINE => &SINH,
        COSINE => &COSH,
        _ => &TANH,
    }
}

/// Normalize a nonzero integer at scale 2^(n-127), carrying the gate's scale.
#[inline]
fn normalized(n: i32, r: u128, gate: u128) -> (i32, u128, u128) {
    let lz = r.leading_zeros();
    (n - lz as i32, r << lz, gate << lz)
}

/// The direct polynomial's entire evaluation error is under eight units of
/// 2^-127 of the result; 32 units include the required twofold margin.
#[inline]
fn small<const KIND: u8>(m: u128, e: i32) -> (i32, u128, u128) {
    let a = m << 15;
    let u = mhi(a, a) >> (-2 * e - 2);
    let c = coefficients::<KIND>();
    let terms = if e < -16 {
        5
    } else if KIND == TANGENT {
        15
    } else {
        10
    };
    let mut p = c[terms - 1][2];
    // Integer fixed-point kernels cannot use the floating-point poly helper.
    for v in c[..terms - 1].iter().rev() {
        let t = mhi(u, p);
        p = if KIND == TANGENT { v[2] - t } else { v[2] + t };
    }
    let correction = mhi(u, p) >> 1;
    let g = if KIND == TANGENT {
        ONE - correction
    } else {
        ONE + correction
    };
    if KIND == COSINE {
        return (0, g, 32);
    }
    let (hi, lo) = wmul(a, g);
    if hi & ONE != 0 {
        (e + 1, hi, 32)
    } else {
        normalized(e, hi << 1 | lo >> 127, 32)
    }
}

/// Raw value and bound seen by the public gate and its MPFR certification.
#[inline]
fn fast<const KIND: u8>(m: u128, e: i32) -> (i32, u128, u128) {
    if e < DIRECT {
        return small::<KIND>(m, e);
    }
    let (n, f) = exp::frame(m, e, &LOG2E, false);
    let (np, p) = exp::fast(n, f);
    if KIND != TANGENT && np >= 64 {
        // exp(-x)/exp(x) < 2^-128, inside one unit of this gate.
        return (np - 1, p, exp::ZIV_GATE + 1);
    }
    let (n, f) = exp::frame(m, e, &LOG2E, true);
    let (nq, q) = exp::fast(n, f);
    let half_p = p >> 1;
    let half_q = q.checked_shr((np - nq + 1) as u32).unwrap_or(0);
    if KIND != TANGENT {
        let r = if KIND == COSINE {
            half_p + half_q
        } else {
            half_p - half_q
        };
        return normalized(np, r, exp::ZIV_GATE + 2);
    }
    let sum = half_p + half_q;
    let diff = half_p - half_q;
    let ls = sum.leading_zeros();
    let ld = diff.leading_zeros();
    let (r, eq) = quotient(diff << ld, sum << ls);
    // Both input errors propagate through n/d; normalization magnifies the
    // numerator by 2^ld. The quotient's own error is under two units.
    (eq - 1 + ls as i32 - ld as i32, r, 128 << ld)
}

fn normalized_384(n: i32, r: [u128; 3]) -> (i32, [u128; 3]) {
    let lz = leading_zeros_384(r);
    (n - lz as i32, shl_384(r, lz))
}

fn small_384<const KIND: u8>(m: u128, e: i32) -> (i32, [u128; 3]) {
    let (hi, lo) = wmul(m, m);
    let u = shl_384([lo, hi, 0], (2 * e + 160) as u32);
    let c = coefficients::<KIND>();
    let mut p = c[c.len() - 1];
    for v in c[..c.len() - 1].iter().rev() {
        let t = mul_hi_384(u, p);
        p = if KIND == TANGENT {
            sub_384(*v, t)
        } else {
            add_384(*v, t)
        };
    }
    let correction = shr_384_sat(mul_hi_384(u, p), 1);
    let g = if KIND == TANGENT {
        sub_384([0, 0, ONE], correction)
    } else {
        add_384([0, 0, ONE], correction)
    };
    if KIND == COSINE {
        return (0, g);
    }
    let p = wmul_128x384(m << 15, g);
    if p[3] & ONE != 0 {
        (e + 1, [p[1], p[2], p[3]])
    } else {
        normalized_384(
            e,
            [
                p[0] >> 127 | p[1] << 1,
                p[1] >> 127 | p[2] << 1,
                p[2] >> 127 | p[3] << 1,
            ],
        )
    }
}

/// Product at scale 2^383, where the exact product is below two.
fn mul383(a: [u128; 3], b: [u128; 3]) -> [u128; 3] {
    shl_384(mul_hi_384(a, b), 1)
}

/// The expq table engine lifted to 384 bits. The complete 380-bit reduction
/// constant already exists; the new tables are generated mathematical values.
fn exponential_384(m: u128, e: i32, negative: bool) -> (i32, [u128; 3]) {
    let product = wmul_128x384(m, [LOG2E.tail, LOG2E.head[0], LOG2E.head[1]]);
    let y = shr_sat(product, (108 - e) as u32);
    let mut n = y[3] as i32;
    let mut f = [y[0], y[1], y[2]];
    if negative {
        n = -n - i32::from(f != [0; 3]);
        f = neg_384(f);
    }
    exp2_384(n, f)
}

/// Evaluate `2^(n + f/2^384)` without rounding to binary128. The returned
/// significand is normalized at 2^383 and paired with its binary exponent.
/// Callers retain their own reduction error in the final rounding bound.
pub(super) fn exp2_384(n: i32, f: [u128; 3]) -> (i32, [u128; 3]) {
    let (i0, i1, i2) = (
        (f[2] >> 122) as usize,
        (f[2] >> 116) as usize & 63,
        (f[2] >> 110) as usize & 63,
    );
    let t = [f[0], f[1], f[2] & ((1 << 110) - 1)];
    let mut q = EXP[19];
    for c in EXP[..19].iter().rev() {
        q = add_384(*c, mul_hi_384(t, q));
    }
    let p = add_384([0, 0, ONE], shr_384_sat(mul_hi_384(t, q), 1));
    let r = mul383(mul383(T1[i1], T2[i2]), p);
    let r = mul_hi_384(T0[i0], r);
    if r[2] & ONE != 0 {
        (n + 1, r)
    } else {
        (n, shl_384(r, 1))
    }
}

fn wide<const KIND: u8>(m: u128, e: i32) -> (i32, [u128; 3]) {
    if e < DIRECT {
        return small_384::<KIND>(m, e);
    }
    let (np, p) = exponential_384(m, e, false);
    if KIND != TANGENT && np >= 192 {
        return (np - 1, p);
    }
    let (nq, q) = exponential_384(m, e, true);
    let p = shr_384_sat(p, 1);
    let q = shr_384_sat(q, (np - nq + 1) as u32);
    if KIND != TANGENT {
        return normalized_384(
            np,
            if KIND == COSINE {
                add_384(p, q)
            } else {
                sub_384(p, q)
            },
        );
    }
    let (ns, sum) = normalized_384(np, add_384(p, q));
    let (nd, diff) = normalized_384(np, sub_384(p, q));
    let (r, eq) = quotient_384(diff, sum);
    (eq - 1 + nd - ns, r)
}

#[cold]
#[inline(never)]
fn accurate<const KIND: u8>(m: u128, e: i32) -> f128 {
    let (n, r) = wide::<KIND>(m, e);
    exp::round(n, r[2], r[1] | u128::from(r[0] != 0))
}

#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::*;
    use rug::{Float, ops::Pow};

    fn mix(mut i: u64) -> u64 {
        i = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        i = (i ^ (i >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        i = (i ^ (i >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        i ^ (i >> 31)
    }
    fn certify<const KIND: u8>() {
        let mut worst = [0.0_f64; 2];
        let mut at = [0.0_f128; 2];
        let mut wide_worst = 0.0_f64;
        for i in 0..1_000_000_u64 {
            let b = u128::from(mix(i)) | u128::from(mix(i ^ 0xCAFE)) << 64;
            let top = if KIND == TANGENT { 5 } else { 13 };
            let e = match i % 4 {
                0 => -57 + (i / 4 % (top + 58) as u64) as i32,
                1 => -4 + (i / 4 % (top + 5) as u64) as i32,
                2 => [-57, -56, -17, -16, -5, -4][(i / 4 % 6) as usize],
                _ => -57 + (i / 4 % 53) as i32,
            };
            let x = f128::from_bits(((BIAS + e) as u128) << 112 | b & ((1 << 112) - 1));
            let (m, e) = split(x.to_bits());
            let (n, r, gate) = fast::<KIND>(m, e);
            let z = Float::with_val(448, x);
            let truth = match KIND {
                SINE => z.sinh(),
                COSINE => z.cosh(),
                _ => z.tanh(),
            };
            let unit = Float::with_val(448, 2).pow(n - 127);
            let error = (Float::with_val(448, &truth / unit) - r).abs().to_f64() / gate as f64;
            let band = usize::from(e >= DIRECT);
            if error > worst[band] {
                worst[band] = error;
                at[band] = x;
            }
            // Certify the accurate precision policy on the same bands, also
            // forcing the fallback at inputs whose fast gate usually passes.
            if i % 128 == 0 {
                let (n, r) = wide::<KIND>(m, e);
                let raw = Float::with_val(448, r[2]) * Float::with_val(448, 2).pow(256)
                    + Float::with_val(448, r[1]) * Float::with_val(448, 2).pow(128)
                    + r[0];
                let actual: Float = raw * Float::with_val(448, 2).pow(n - 383);
                let ratio: Float =
                    ((actual - &truth) / &truth).abs() * Float::with_val(448, 2).pow(360);
                wide_worst = wide_worst.max(ratio.to_f64());
            }
        }
        println!(
            "hyperbolic {KIND}: worst |err|/gate {worst:?} at {at:?}; wide |relative err|/2^-360 = {wide_worst:e}"
        );
        assert!(worst.into_iter().all(|v| v < 0.5));
        assert!(wide_worst < 0.5);
    }
    #[test]
    fn sinhq_fast_legs_are_sound() {
        certify::<SINE>();
    }
    #[test]
    fn coshq_fast_legs_are_sound() {
        certify::<COSINE>();
    }
    #[test]
    fn tanhq_fast_legs_are_sound() {
        certify::<TANGENT>();
    }
}
