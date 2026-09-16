//! Binary128 trigonometric functions in half-turns.
//!
//! `N = round(256·|x|)` and `r = |x| − N/256` are exact integer cuts of
//! the input significand; the table index keeps only `n = N mod 512`. The nonzero residual normalizes without
//! losing a bit, then multiplies the existing π/2 limbs (with one extra
//! exponent bit) before entering the sine/cosine or tangent kernels. There
//! is no Payne–Hanek window and no cancellation in the multiplication by π.
//!
//! Integer and half-integer sine/cosine values, and quarter-integer tangent
//! values, return directly. These are all the dyadic exact cases: a rational
//! sine or cosine of a rational multiple of π is 0, ±1/2, or ±1, with ±1/2
//! requiring a nondyadic third; rational tangent values are 0 and ±1.
//! The accurate leg reuses the 384-bit kernels; its π multiplication is
//! accurate to better than 2^-381, below the kernels' own series error.

use super::atan2::{round_fast, top_256};
use super::trig::{self, Residual};
use super::trig_tables::{PIO2_128, PIO2_384};
use super::uint::{mul_hi_384, shl_384, wmul};
use super::{BIAS, EXP_MASK, EXP_SHIFT, SIGN_MASK, split, tan};

/// Sine of π times `x`, rounded to nearest, ties to even.
///
/// Integers return zero with the sign of `x`; half-integers return ±1.
/// Infinite arguments return NaN.
#[must_use]
pub fn sinpiq(x: f128) -> f128 {
    trigpi(x, 0)
}

/// Cosine of π times `x`, rounded to nearest, ties to even.
///
/// Integers return ±1; half-integers return positive zero.
/// Infinite arguments return NaN.
#[must_use]
pub fn cospiq(x: f128) -> f128 {
    trigpi(x, 1)
}

/// Tangent of π times `x`, rounded to nearest, ties to even.
///
/// For nonnegative integers `n`, `tanpiq(n)` is +0 for even `n`, −0 for
/// odd `n`, and `tanpiq(n + 1/2)` is +∞ for even `n`, −∞ for odd `n`.
/// Negative arguments follow odd symmetry, including −0. Quarter-integers
/// return ±1 exactly. Infinite arguments return NaN.
#[must_use]
pub fn tanpiq(x: f128) -> f128 {
    trigpi(x, 2)
}

#[inline]
fn trigpi(x: f128, kind: u8) -> f128 {
    let bits = x.to_bits();
    let ax = bits & !SIGN_MASK;
    if ax >= EXP_MASK {
        return trig::edge(bits, ax);
    }
    if ax == 0 {
        return if kind == 1 { 1.0 } else { x };
    }
    // (πx)²/2 < 2^-115.6, below the 2^-114 half-ulp immediately below 1.
    if kind == 1 && ax < ((BIAS - 59) as u128) << EXP_SHIFT {
        return 1.0;
    }
    let sign = if kind == 1 { 0 } else { bits & SIGN_MASK };
    let (m, e) = split(ax);
    let r = reduce(m, e);
    if let Some(y) = exact(&r, kind, sign) {
        return y;
    }
    fast(&r, kind)
        .and_then(|(f, e, flip)| round_fast(f, e, sign ^ flip, gate(kind)))
        .unwrap_or_else(|| accurate(&r, kind, sign))
}

/// Exact `|x| = n/256 + r` modulo 2, `|r| = f·2^(e−128)`.
struct Reduction {
    n: usize,
    negative: bool,
    f: u128,
    e: i32,
}

#[inline]
fn reduce(m: u128, e: i32) -> Reduction {
    if e >= 104 {
        // All inputs are on the table grid; above 2^113 they are even integers.
        let n = if e >= 113 {
            0
        } else {
            (m << (e - 104)) as usize & 511
        };
        return Reduction {
            n,
            negative: false,
            f: 0,
            e: 0,
        };
    }
    if e < -9 {
        return Reduction {
            n: 0,
            negative: false,
            f: m << 15,
            e: e + 1,
        };
    }
    let shift = (104 - e) as u32; // 1..=113
    let half = 1u128 << (shift - 1);
    let n = ((m + half) >> shift) as usize & 511;
    let tail = m & ((1 << shift) - 1);
    let negative = tail >= half;
    let f = if negative { (1 << shift) - tail } else { tail };
    let lz = f.leading_zeros().min(127);
    Reduction {
        n,
        negative,
        f: f << lz,
        e: e + 16 - lz as i32,
    }
}

/// Every dyadic exact case, including the IEEE tanPi zero/pole parity.
#[inline]
fn exact(r: &Reduction, kind: u8, sign: u128) -> Option<f128> {
    if r.f != 0 {
        return None;
    }
    let n = r.n;
    if kind == 2 {
        if n & 63 != 0 {
            return None;
        }
        let (magnitude, flip) = if n & 64 != 0 {
            (1.0_f128.to_bits(), ((n & 128) as u128) << 120)
        } else {
            (
                if n & 128 != 0 { EXP_MASK } else { 0 },
                ((n & 256) as u128) << 119,
            )
        };
        return Some(f128::from_bits(magnitude | (sign ^ flip)));
    }
    if n & 127 != 0 {
        return None;
    }
    let k = (n >> 7) + usize::from(kind == 1);
    Some(if k & 1 == 0 {
        f128::from_bits(sign)
    } else {
        f128::from_bits(1.0_f128.to_bits() | (sign ^ (((k & 2) as u128) << 126)))
    })
}

#[inline]
const fn gate(kind: u8) -> u128 {
    if kind == 2 {
        super::atan2::ZIV_GATE
    } else {
        trig::ZIV_GATE
    }
}

/// Scale the exact residual by π, then use the unchanged trig kernels.
#[inline]
fn fast(r: &Reduction, kind: u8) -> Option<(u128, i32, u128)> {
    let (t1, et) = if r.f == 0 {
        (0, -32) // Exact table entry; every correction vanishes.
    } else {
        let (hi, lo) = wmul(r.f, PIO2_128);
        let lz = hi.leading_zeros();
        (top_256([lo, hi], lz), r.e + 2 - lz as i32)
    };
    if et < -63 {
        // Below 2^-63, the relative cubic correction is < 2^-127.5.
        // Only the zero breakpoint uses this shortcut; tiny residuals about other
        // breakpoints go to the accurate leg, beyond the kernels' shift range.
        return (r.n == 0 && kind != 1).then_some((t1, et, if r.negative { SIGN_MASK } else { 0 }));
    }
    let r = Residual {
        n: r.n,
        negative: r.negative,
        t1,
        et,
    };
    Some(if kind == 2 {
        tan::fast_reduced(r)
    } else {
        trig::fast_reduced(r, kind == 1)
    })
}

/// Recompute π·r at 384 bits, preserving relative accuracy through subnormals.
#[cold]
#[inline(never)]
fn accurate(r: &Reduction, kind: u8, sign: u128) -> f128 {
    let (t, et) = if r.f == 0 {
        ([0; 3], -32)
    } else {
        let t = mul_hi_384([0, 0, r.f], PIO2_384);
        let lz = t[2].leading_zeros();
        (shl_384(t, lz), r.e + 2 - lz as i32)
    };
    if kind == 2 {
        tan::accurate_reduced(r.n, r.negative, t, et, sign)
    } else {
        trig::accurate_reduced(r.n, r.negative, t, et, kind == 1, sign)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specials() {
        for f in [sinpiq, cospiq, tanpiq] {
            for x in [f128::INFINITY, f128::NEG_INFINITY, f128::NAN] {
                assert!(f(x).is_nan());
            }
            let nan = SIGN_MASK | EXP_MASK | 123;
            assert_eq!(
                f(f128::from_bits(nan)).to_bits(),
                nan | super::super::QUIET_BIT
            );
        }
        assert_eq!(sinpiq(-0.0).to_bits(), SIGN_MASK);
        assert_eq!(tanpiq(-0.0).to_bits(), SIGN_MASK);
        assert_eq!(cospiq(-0.0), 1.0);
        for n in 0..=128 {
            let x = f128::from(n);
            let s = if n & 1 == 0 { 1.0 } else { -1.0 };
            for sign in [1.0, -1.0] {
                assert_eq!(sinpiq(sign * x).to_bits(), (sign * 0.0_f128).to_bits());
                assert_eq!(cospiq(sign * x), s);
                assert_eq!(tanpiq(sign * x).to_bits(), (sign * s * 0.0_f128).to_bits());
                assert_eq!(sinpiq(sign * (x + 0.5)), sign * s);
                assert_eq!(cospiq(sign * (x + 0.5)).to_bits(), 0);
                assert_eq!(tanpiq(sign * (x + 0.5)), sign * s * f128::INFINITY);
                assert_eq!(tanpiq(sign * (x + 0.25)), sign);
                assert_eq!(tanpiq(sign * (x + 0.75)), -sign);
            }
        }
        for x in [super::super::exp2i(113), f128::MAX] {
            assert_eq!(sinpiq(x).to_bits(), 0);
            assert_eq!(sinpiq(-x).to_bits(), SIGN_MASK);
            assert_eq!(tanpiq(x).to_bits(), 0);
            assert_eq!(tanpiq(-x).to_bits(), SIGN_MASK);
            assert_eq!(cospiq(x), 1.0);
            assert_eq!(cospiq(-x), 1.0);
        }
        let least = f128::from_bits(1);
        assert_eq!(sinpiq(least).to_bits(), 3);
        assert_eq!(tanpiq(least).to_bits(), 3);
        assert_eq!(sinpiq(-least).to_bits(), SIGN_MASK | 3);
        assert_eq!(tanpiq(-least).to_bits(), SIGN_MASK | 3);
        assert_eq!(cospiq(least), 1.0);
    }
}

/// Certify the complete π-scaled fast leg, including multiplication by π,
/// table reconstruction, the tangent quotient, and the tiny linear band.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::*;
    use rug::{Float, float::Round};

    const PREC: u32 = 300;

    fn mix(mut i: u64) -> u64 {
        i = (i ^ (i >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        i = (i ^ (i >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        i ^ (i >> 31)
    }

    fn draw(i: u64) -> f128 {
        let bits = u128::from(mix(i)) | u128::from(mix(i ^ 0x9e37_79b9_7f4a_7c15)) << 64;
        let m = bits & ((1 << 112) - 1);
        match i % 5 {
            0 => f128::from_bits((bits >> 112) % 0x7fff << 112 | m),
            1 => {
                let e = BIAS - 67 + (bits >> 112) as i32 % 180;
                f128::from_bits((e as u128) << 112 | m)
            }
            2 => {
                let x = ((bits >> 64) % 1024) as f128 / 256.0;
                let d = super::super::exp2i(-9 - (bits % 120) as i64);
                if bits & 1 == 0 { x + d } else { (x - d).abs() }
            }
            3 => {
                let x = ((bits >> 64) % 1024 + 1) as f128 / 512.0;
                f128::from_bits(x.to_bits().wrapping_add_signed((bits % 7) as i128 - 3))
            }
            _ => f128::from_bits(((BIAS - 67 + (bits >> 112) as i32 % 10) as u128) << 112 | m),
        }
    }

    #[test]
    fn fast_legs_are_sound() {
        let mut worst = [0.0_f64; 3];
        let mut worst_at = [0.0_f128; 3];
        let mut handed_over = [0; 3];
        for i in 1..=1_000_000 {
            let x = draw(i);
            if x == 0.0 {
                continue;
            }
            let (m, e) = split(x.to_bits());
            let r = reduce(m, e);
            for kind in 0..3 {
                if exact(&r, kind, 0).is_some() {
                    continue;
                }
                let Some((f, e, flip)) = fast(&r, kind) else {
                    handed_over[usize::from(kind)] += 1;
                    continue;
                };
                let mut truth = Float::with_val(PREC, x);
                match kind {
                    0 => truth.sin_pi_round(Round::Nearest),
                    1 => truth.cos_pi_round(Round::Nearest),
                    _ => truth.tan_pi_round(Round::Nearest),
                };
                assert_eq!(flip != 0, truth.is_sign_negative(), "kind={kind}, x={x:?}");
                // Express the error in the normalized frame's gate units.
                truth.abs_mut();
                truth <<= 128 - e;
                truth -= Float::with_val(PREC, f);
                let ratio = truth.abs().to_f64() / gate(kind) as f64;
                let k = usize::from(kind);
                if ratio > worst[k] {
                    worst[k] = ratio;
                    worst_at[k] = x;
                }
            }
        }
        for k in 0..3 {
            println!(
                "{} fast leg: worst |err|/gate = {:.6} at {:?} ({} handed over)",
                ["sinpiq", "cospiq", "tanpiq"][k],
                worst[k],
                worst_at[k],
                handed_over[k]
            );
            assert!(worst[k] < 0.5, "kind={k}: only {}× margin", 1.0 / worst[k]);
        }
    }
}
