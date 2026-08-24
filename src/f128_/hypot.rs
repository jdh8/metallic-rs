use super::roots::sqrt_wide;
use super::uint::{add_384, cmp_384, extract_u128, leading_zeros_384, mhi, shl_384, sqr_384, wmul};
use super::{BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, QUIET_BIT, SIGN_MASK, split};
use core::cmp::Ordering;

/// Half-width, in units of the candidate's 2^-125, of the rounding-tie window
/// [`hypotq`] refuses to decide from the fast candidate and hands to the exact
/// 384-bit walk instead.  [`hypot_fixed`] is analytically within a handful of
/// units of the true norm; [`ziv_soundness`] certifies the ≥ 2× margin the
/// project requires.
const HYPOT_GATE: u128 = 32;

/// The Euclidean norm, √(x² + y²).
///
/// The fast leg is [`sqrtq`](super::roots::sqrtq)'s fixed-point frame applied
/// to the truncated 128-bit sum of squares from [`hypot_fixed`], rounding on
/// 13 guard bits.  The [`HYPOT_GATE`] tie window falls back to the exact tier:
/// with the exponent gap capped at 56, the sum of squares `ma² · 2^(2·dn) +
/// mb²` is an *exact* 384-bit integer `V`, and the last bit follows from two
/// exact comparisons against it — one pinning the significand to `⌊√V⌋`, one
/// deciding the half-way test.
#[must_use]
pub fn hypotq(x: f128, y: f128) -> f128 {
    let (p, q) = (x.to_bits() & !SIGN_MASK, y.to_bits() & !SIGN_MASK);
    let (a, b) = if p >= q { (p, q) } else { (q, p) };

    // Sign-magnitude order puts NaN above ∞ above every finite magnitude, so
    // only the larger operand needs the special test.
    if a >= EXP_MASK {
        if is_signaling(a) || is_signaling(b) {
            return f128::NAN;
        }
        // hypot(±∞, y) is +∞ for every y, a quiet NaN included.
        if a == EXP_MASK || b == EXP_MASK {
            return f128::INFINITY;
        }
        return f128::from_bits(a);
    }
    if b == 0 {
        return f128::from_bits(a);
    }

    let (ma, ea) = split(a);
    let (mb, eb) = split(b);

    // With `b < 2^(eb+1)` and `|a| ≥ 2^ea`, the excess `√(a² + b²) − |a| ≤
    // b²/2|a|` stays below `2^(2·eb + 1 − ea)`, which for `dn ≥ 57` is at most
    // `2^(ea − 113) ≤ ½ ulp(|a|)` — so the sum rounds straight back to `|a|`.
    // The same cutoff caps the shift in [`exact`] at 112 bits, keeping V
    // inside the 384-bit window.
    if ea - eb > 56 {
        return f128::from_bits(a);
    }
    #[allow(clippy::cast_sign_loss)]
    let dn = (ea - eb) as u32;

    let (candidate, carry) = hypot_fixed(ma, mb, dn);
    let e = ea + carry as i32;
    // `carry` never overshoots (the sum of squares is truncated *down*), so
    // `e > MAX_EXP − 1` proves the result at least 2^16384, well past the
    // overflow threshold.
    if e > f128::MAX_EXP - 1 {
        return f128::INFINITY;
    }
    // The exact tier is total: it also owns the subnormal grid and the rare
    // case where truncation hid a carry at the bottom of the normal range.
    let rest = candidate & 8191;
    if e < f128::MIN_EXP - 1 || rest.abs_diff(4096) <= HYPOT_GATE {
        return exact(ma, mb, dn, eb);
    }
    // Round the Q125 candidate to 113 bits.  Composing the bits by adding the
    // significand onto a BIAS − 1 exponent field lets a carry out of the
    // rounding land on an exact 2 instead of overflowing the mantissa — and at
    // `e = MAX_EXP − 1` that carry composes ∞, which is then the correct
    // rounding of a result beyond 2^16384·(1 − 2^-114).
    f128::from_bits((((BIAS - 1 + e) as u128) << EXP_SHIFT) + ((candidate + 4096) >> 13))
}

/// `(√(z/4^carry)·2^125, carry)` for `z = (ma·2^-112)² + (mb·2^(-dn-112))² ∈
/// [1, 8)`, the candidate within [`HYPOT_GATE`]/2 units of 2^-125.
///
/// The truncations are one-sided: `B` floors `mb·2^(14−dn)` (costing `v` up to
/// two units through the cross term) and the discarded low half of the sum of
/// squares up to one more, so `v ∈ [z·2^124 − 3, z·2^124]`.  `carry` reads the
/// octave off the truncated `v`, which therefore never overshoots the true
/// octave; a truncation *across* 2^126 only re-labels `z ≈ 4` into the `[1, 4)`
/// frame, where the rounding composition in [`hypotq`] reunifies the two
/// descriptions of the same real number.
fn hypot_fixed(ma: u128, mb: u128, dn: u32) -> (u128, u32) {
    let big = ma << 14;
    let small = (mb << 14) >> dn;
    let (high_a, low_a) = wmul(big, big);
    let (high_b, low_b) = wmul(small, small);
    let (_, spill) = low_a.overflowing_add(low_b);
    let v = high_a + high_b + u128::from(spill);

    // v ∈ [2^124, 2^127) spans three octaves; fold `z ≥ 4` down by reading v
    // as (z/4)·2^126 directly, so the carry case loses no bits at all.
    #[allow(clippy::cast_possible_truncation)]
    let carry = (v >> 126) as u32;
    (sqrt_wide(v << (2 - 2 * carry)), carry)
}

/// Whether a bit pattern is a NaN without the quiet bit.
#[inline]
const fn is_signaling(bits: u128) -> bool {
    bits > EXP_MASK && bits & QUIET_BIT == 0
}

/// The exact tier: round `√(ma²·2^(2·dn) + mb²)·2^(eb−112)` by integer
/// comparisons against the exact 384-bit sum of squares.
fn exact(ma: u128, mb: u128, dn: u32, eb: i32) -> f128 {
    let v = add_384(shl_384(sqr_384(ma), 2 * dn), sqr_384(mb));

    // √V spans ⌈L/2⌉ bits for an L-bit V.  Writing h for that count, the result
    // is `c · 2^(e − 112)` with `c = √V · 2^(113 − h)` in `[2^112, 2^113)`, and
    // `c² · 2^t` compares directly against V.
    let l = 384 - leading_zeros_384(v);
    let h = l.div_ceil(2);
    let e = h as i32 - 113 + eb;
    let t = 2 * h - 226;

    // Seed `q ≈ 2^190/√w` from f64, where w carries V's leading 127-odd bits at
    // an *even* shift, which makes `c` exactly `√w · 2^49`.  Two Newton steps
    // take the 53-bit seed past 120 bits, so the fixed-point truncations, not
    // the iteration, set the residual error.
    let w = extract_u128(v, l - 128 + (l & 1));
    let seed = crate::exp2i(158) / ((w >> 64) as u64 as f64).sqrt();
    let q = rsqrt_step(w, rsqrt_step(w, seed as u128));
    let mut c = mhi(w, q) >> 13;

    // Pin c to ⌊√(V · 2^−t)⌋.  An exhaustive scan of the adversarial mantissa
    // corners plus 300k random ones puts the seed within one unit either way,
    // so each loop runs at most one iteration.
    while cmp_384(shl_384(sqr_384(c), t), v) == Ordering::Greater {
        c -= 1;
    }
    while cmp_384(shl_384(sqr_384(c + 1), t), v) != Ordering::Greater {
        c += 1;
    }

    // The result is now `(c + f) · 2^(e − 112)` with `f` in `[0, 1)`.
    if e < f128::MIN_EXP - 1 {
        return subnormal(v, c, t, e);
    }
    if e > f128::MAX_EXP - 1 {
        return f128::INFINITY;
    }

    // `f > ½ ⟺ 4V > (2c + 1)² · 2^t`; equality is a tie, broken to even.  The
    // round bit is added to the *packed* form, so `c = 2^113 − 1` carries into
    // the exponent and `MAX` carries into `+∞`.
    let side = cmp_384(shl_384(v, 2), shl_384(sqr_384(2 * c + 1), t));
    let round = side == Ordering::Greater || (side == Ordering::Equal && c & 1 != 0);
    let packed = ((e + BIAS) as u128) << EXP_SHIFT | (c - IMPLICIT_BIT);

    f128::from_bits(packed + u128::from(round))
}

/// One Newton step for `q ≈ 2^190/√w`.
///
/// Scaling q by 2^126 and w by 2^128 turns `q ← q(3 − w q²)/2` into a pair of
/// high multiplies against the fixed point 2^124: `w q²` lands there, and the
/// residual times `q/2` is the correction.
#[inline]
fn rsqrt_step(w: u128, q: u128) -> u128 {
    const UNIT: u128 = 1 << 124;
    let residual = mhi(w, mhi(q, q));

    if residual >= UNIT {
        q - (mhi(q, residual - UNIT) << 3)
    } else {
        q + (mhi(q, UNIT - residual) << 3)
    }
}

/// Round `(c + f) · 2^(e − 112)`, `f` in `[0, 1)`, onto the fixed 2^−16494
/// subnormal grid, where `f` is zero exactly when `c² · 2^t` equals V.
///
/// An exact tie cannot actually happen down here — both legs are subnormal, so
/// a halfway result would need `4(A² + B²)` to be an odd square — but keeping
/// the parity term leaves round-to-nearest-ties-to-even stated in full.
fn subnormal(v: [u128; 3], c: u128, t: u32, e: i32) -> f128 {
    let shift = (f128::MIN_EXP - 1 - e) as u32;
    let half = 1 << (shift - 1);
    let rest = c & ((1 << shift) - 1);
    let n = c >> shift;
    let round = rest > half
        || rest == half && (n & 1 != 0 || cmp_384(shl_384(sqr_384(c), t), v) != Ordering::Equal);

    f128::from_bits(n + u128::from(round))
}

/// MPFR certification that [`HYPOT_GATE`] covers the true error of
/// [`hypot_fixed`] with the 2× margin the project requires.  Run with
/// `CC=clang cargo +nightly test --release --features "f128 mpfr"`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::super::MANTISSA_MASK;
    use super::*;
    use rug::{Float, Integer, ops::Pow};

    const PRECISION: u32 = 300;
    const SAMPLES: u64 = 200_000;

    fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Worst `|candidate − √(z/4^carry)·2^125| / HYPOT_GATE` over random
    /// significand pairs and exponent gaps — exactly the units the gate
    /// compares.  Truth uses the leg's own `carry`, matching the exponent the
    /// rounding composition assigns to the candidate.
    #[test]
    fn fast_candidate_is_sound() {
        let mut worst = 0.0;
        let mut worst_at = (0, 0, 0);

        for i in 0..SAMPLES {
            let mut ma = (u128::from(mix(i)) << 64 | u128::from(mix(i ^ 0x9E37_79B9)))
                & MANTISSA_MASK
                | IMPLICIT_BIT;
            let mut mb = (u128::from(mix(i ^ 0x5DEE_CE66)) << 64 | u128::from(mix(!i)))
                & MANTISSA_MASK
                | IMPLICIT_BIT;
            // Low gaps dominate the error (B's truncation shrinks with dn), so
            // spend most samples there while still covering the full range.
            let dn = match mix(i ^ 0xABCD) % 8 {
                r @ 0..4 => r as u32,
                4..6 => (mix(i ^ 0xEF01) % 8) as u32,
                _ => (mix(i ^ 0xEF01) % 57) as u32,
            };
            if dn == 0 && mb > ma {
                core::mem::swap(&mut ma, &mut mb);
            }
            let (candidate, carry) = hypot_fixed(ma, mb, dn);

            // √S·2^(13 − dn − carry) = √(z/4^carry)·2^125 for the exact
            // integer S = ma²·2^(2·dn) + mb².
            let s = (Integer::from(ma) * ma << (2 * dn)) + Integer::from(mb) * mb;
            let truth = Float::with_val(PRECISION, s).sqrt()
                * Float::with_val(PRECISION, 2).pow(13 - dn as i32 - carry as i32);
            let slip: Float = truth - Float::with_val(PRECISION, candidate);
            let ratio = slip.abs().to_f64() / HYPOT_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_at = (ma, mb, dn);
            }
        }
        println!(
            "hypotq fast candidate: worst |err|/gate = {worst:.4} at ma={:#x} mb={:#x} dn={}",
            worst_at.0, worst_at.1, worst_at.2
        );
        assert!(
            worst < 0.5,
            "hypotq gate covers only {:.2}× the slip at ma={:#x} mb={:#x} dn={}",
            1.0 / worst,
            worst_at.0,
            worst_at.1,
            worst_at.2
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pythagorean_and_specials() {
        assert_eq!(hypotq(3.0, -4.0).to_bits(), 5.0_f128.to_bits());
        assert_eq!(hypotq(-0.0, -0.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(hypotq(-1.0, 0.0).to_bits(), 1.0_f128.to_bits());
        assert_eq!(
            hypotq(f128::NEG_INFINITY, f128::NAN).to_bits(),
            f128::INFINITY.to_bits()
        );
        assert!(hypotq(f128::NAN, 1.0).is_nan());
        assert!(hypotq(f128::MAX, f128::MAX).is_infinite());
        // Both legs subnormal: a 3-4-5 triple on the 2^-16494 grid.
        assert_eq!(hypotq(f128::from_bits(3), f128::from_bits(4)).to_bits(), 5);
    }
}
