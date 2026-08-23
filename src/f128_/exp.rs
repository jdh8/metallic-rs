//! The binary128 exponential family, all reduced to one engine for 2<sup>y</sup>.
//!
//! `expq`, `exp2q` and `exp10q` differ only in the constant `L` of
//! `f(x) = 2^(x·L)`, so they share the whole pipeline:
//!
//! 1. **Reduce.** `y = x·L` is formed as an exact 384-bit integer product of the
//!    113-bit significand with a 256-bit `L`, then shifted into a fixed frame:
//!    an integer part `n` and a 256-bit fraction `f ∈ [0, 1)`.  Two's
//!    complement handles `x < 0`, so `n = ⌊y⌋` and `f = y − n` come out of the
//!    same code path for both signs.
//! 2. **Look up.** The leading 18 bits of `f` index three 64-entry tables of
//!    2<sup>j/64</sup>, 2<sup>j/4096</sup> and 2<sup>j/262144</sup>, leaving
//!    `t < 2^-18` for the polynomial.
//! 3. **Evaluate.** `2^t − 1 = Σ ln(2)^k/k! · t^k` converges so fast at that
//!    width that the seventh term already sits below 2^-142.
//! 4. **Round.** The product of the three table entries with `2^t` is `2^f`,
//!    which is exactly `[1, 2)`, so no renormalization is needed except for the
//!    boundary carry.  `n` becomes the exponent field.
//!
//! Everything is unsigned fixed point: an `f128` multiply is soft-float on every
//! target that has no hardware binary128, so integer limbs are both faster and
//! exactly analyzable.  The fast leg carries 128 bits, 15 more than binary128
//! needs; when the discarded 15 bits sit within [`ZIV_GATE`] of a rounding tie
//! the accurate leg redoes the same steps at 256 bits.

use super::exp_tables::{COEF, LOG2_10, LOG2E, ONE, Reduction, T0, T1, T2};
use super::uint::{add_256, add_384, mhi, mul_hi_256, neg_384, wmul};
use super::{BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, QUIET_BIT, SIGN_MASK, split};

/// Half-width of the rounding-tie window the fast leg refuses to decide.
///
/// The fast leg's significand carries at most eight units of 2^-128 of error
/// (three table roundings, three truncated products, and the polynomial), so 32
/// units leave the 2× soundness margin certified in [`ziv_soundness`].
const ZIV_GATE: u128 = 32;

/// `|x| ≥ 2^15` overflows or underflows every member of the family.
const SATURATE: u128 = ((BIAS + 15) as u128) << EXP_SHIFT;

/// Below 2^-120 every member of the family rounds to 1: the largest slope in
/// play is `ln(10) < 2^1.2`, leaving `|f(x) − 1| < 2^-118.8`, well inside the
/// half ulp 2^-114 around 1.
const TINY: u128 = ((BIAS - 120) as u128) << EXP_SHIFT;

/// The exponential function, e<sup>x</sup>.
#[must_use]
pub fn expq(x: f128) -> f128 {
    exp_generic(x, &LOG2E)
}

/// The base-2 exponential function, 2<sup>x</sup>.
#[must_use]
pub fn exp2q(x: f128) -> f128 {
    exp_generic(x, &ONE)
}

/// The base-10 exponential function, 10<sup>x</sup>.
#[must_use]
pub fn exp10q(x: f128) -> f128 {
    exp_generic(x, &LOG2_10)
}

/// 2<sup>x·L</sup> for a reduction constant `L`.
#[inline]
fn exp_generic(x: f128, l: &Reduction) -> f128 {
    let bits = x.to_bits();
    let magnitude = bits & !SIGN_MASK;

    if magnitude >= SATURATE {
        if magnitude > EXP_MASK {
            return f128::from_bits(bits | QUIET_BIT);
        }
        return if bits & SIGN_MASK == 0 {
            f128::INFINITY
        } else {
            0.0
        };
    }
    if magnitude < TINY {
        return 1.0;
    }

    let (m, e) = split(magnitude);
    let negative = bits & SIGN_MASK != 0;
    let y = signed(reduce(m, e, l.head), negative);
    let (n, r) = fast(y[2] as i32, y[1]);

    if undecided(n, r) {
        return accurate(m, e, negative, l);
    }
    round(n, r, 0)
}

/// `|x|·L` as `[fraction low, fraction high, integer part]`, the fraction scaled
/// by 2^-256.
///
/// `head` is `L·2^252` truncated to 256 bits, so the exact product
/// `m · head = |x|·L · 2^(364 − e)` needs only a right shift to land in the
/// frame.  Truncating `L` biases `y` by at most `|y|·2^-252 < 2^-235`.
#[inline]
fn reduce(m: u128, e: i32, head: [u128; 2]) -> [u128; 3] {
    let (high, low) = wmul(m, head[0]);
    let (top, middle) = wmul(m, head[1]);
    let (middle, carry) = high.overflowing_add(middle);
    let product = [low, middle, top + u128::from(carry)];
    let shift = (108 - e) as u32;

    // `94 ≤ shift ≤ 228`, so the frame lands one or two limbs down: pick the
    // window, then funnel the rest.  `|y| < 1` once a whole limb is gone, which
    // is exactly the case whose integer limb is shifted in as zero.
    let [low, middle, high] = if shift < 128 {
        product
    } else {
        [product[1], product[2], 0]
    };
    let bits = shift & 127;

    // `<< 1 << (127 - bits)` is `<< (128 - bits)` with the no-op case in range.
    [
        (low >> bits) | (middle << 1 << (127 - bits)),
        (middle >> bits) | (high << 1 << (127 - bits)),
        high >> bits,
    ]
}

/// Apply the sign of `x` to a reduced `[fraction, integer]` triple.
///
/// Negating in two's complement is exactly the `⌊y⌋` / `y − ⌊y⌋` split for
/// negative `y`, with the borrow moving the integer part down by one.  The
/// complement is folded into a mask rather than a branch: the sign of a random
/// argument is unpredictable, and a mispredict here costs more than the whole
/// negation.
#[inline]
fn signed(y: [u128; 3], negative: bool) -> [u128; 3] {
    if negative { neg_384(y) } else { y }
}

/// The residual `L − head/2^252` of the reduction constant, in units of 2^-256
/// of the fraction.
fn correction(m: u128, e: i32, tail: u128) -> u128 {
    let shift = 236 - e;

    if shift >= 256 {
        0
    } else {
        mhi(m, tail) >> (shift - 128)
    }
}

/// 2<sup>f</sup> to 128 bits, as `(exponent bump, significand scaled by 2^127)`.
#[inline]
fn fast(n: i32, f: u128) -> (i32, u128) {
    let (i0, i1, i2, t) = index(f);

    // The last three terms ride on `t^4 < 2^-72`, so 64-bit coefficients and a
    // 64-bit argument hold them to 2^-136 — a quarter of the multiplier work of
    // the 128-bit steps, which start where that no longer suffices.
    let short = (t >> 64) as u64;
    let tail = coefficient(3) + mul_hi_64(short, coefficient(4) + mul_hi_64(short, coefficient(5)));
    let mut q = COEF[2][1] + mhi(t, u128::from(tail) << 64);

    for c in COEF[..2].iter().rev() {
        q = c[1] + mhi(t, q);
    }
    // `2^t − 1 < 2^-18` at scale 2^-128 becomes `2^t` at scale 2^-127.
    let p = (1 << 127) + (mhi(t, q) >> 1);
    let r = mul127(mul127(T1[i1][1], T2[i2][1]), p);
    let (high, low) = wmul(T0[i0][1], r);
    let carry = high >> 127;

    // `2^f < 2` leaves the product one bit short of the frame, except when
    // rounding pushes it up to the boundary.
    (
        n + carry as i32,
        if carry == 0 {
            (high << 1) | (low >> 127)
        } else {
            high
        },
    )
}

/// [`fast`] at 256 bits, from the reduction up.
#[cold]
#[inline(never)]
fn accurate(m: u128, e: i32, negative: bool, l: &Reduction) -> f128 {
    let y = reduce(m, e, l.head);
    let y = signed(add_384(y, [correction(m, e, l.tail), 0, 0]), negative);
    let (i0, i1, i2, t) = index(y[1]);
    let t = [y[0], t];
    let mut q = COEF[12];

    for c in COEF[..12].iter().rev() {
        q = add_256(*c, mul_hi_256(t, q));
    }
    let p = mul_hi_256(t, q);
    let p = [(p[0] >> 1) | (p[1] << 127), (p[1] >> 1) | 1 << 127];
    let r = mul255(mul255(T1[i1], T2[i2]), p);
    let r = mul_hi_256(T0[i0], r);
    let carry = r[1] >> 127;
    let r = if carry == 0 {
        [r[0] << 1, (r[1] << 1) | (r[0] >> 127)]
    } else {
        r
    };

    round(y[2] as i32 + carry as i32, r[1], r[0])
}

/// The `k`-th Taylor coefficient truncated to 64 bits, scaled by 2^-64.
#[inline]
const fn coefficient(k: usize) -> u64 {
    (COEF[k][1] >> 64) as u64
}

/// High half of an unsigned 64×64-bit product.
#[inline]
const fn mul_hi_64(x: u64, y: u64) -> u64 {
    ((x as u128 * y as u128) >> 64) as u64
}

/// Split the leading 18 bits of a fraction into the three table indices, and
/// return the `t < 2^-18` left for the polynomial.
#[inline]
const fn index(f: u128) -> (usize, usize, usize, u128) {
    (
        (f >> 122) as usize,
        (f >> 116) as usize & 63,
        (f >> 110) as usize & 63,
        f & ((1 << 110) - 1),
    )
}

/// Product of two significands scaled by 2^127, truncated back to that scale.
///
/// Sound only where the exact product stays below 2, which is why the table
/// levels multiply before the leading one.
#[inline]
fn mul127(a: u128, b: u128) -> u128 {
    let (high, low) = wmul(a, b);
    (high << 1) | (low >> 127)
}

/// [`mul127`] at 256 bits.
fn mul255(a: [u128; 2], b: [u128; 2]) -> [u128; 2] {
    let p = mul_hi_256(a, b);
    [p[0] << 1, (p[1] << 1) | (p[0] >> 127)]
}

/// Bits of the significand the final rounding discards, 128 or less.
#[inline]
fn discarded(n: i32) -> u32 {
    (15 + (f128::MIN_EXP - 1 - n).max(0)) as u32
}

/// Whether the fast leg's significand sits too close to a rounding tie to
/// decide the last bit.
#[inline]
fn undecided(n: i32, r: u128) -> bool {
    let shift = discarded(n);

    // Saturating results carry no significand bits to decide.
    if n > f128::MAX_EXP - 1 || shift > 128 {
        return false;
    }
    (r & (u128::MAX >> (128 - shift))).abs_diff(1 << (shift - 1)) <= ZIV_GATE
}

/// Round `(high + low·2^-128)·2^(n − 127)` to binary128, ties to even.
#[inline]
fn round(n: i32, high: u128, low: u128) -> f128 {
    if n > f128::MAX_EXP - 1 {
        return f128::INFINITY;
    }
    let shift = discarded(n);

    // Below half the smallest subnormal every significand rounds to zero.
    if shift > 128 {
        return 0.0;
    }
    let rest = high & (u128::MAX >> (128 - shift));
    let half = 1 << (shift - 1);
    let mantissa = high.checked_shr(shift).unwrap_or(0);
    let up = rest > half || (rest == half && (low != 0 || mantissa & 1 != 0));
    let packed = if n >= f128::MIN_EXP - 1 {
        (((n + BIAS) as u128) << EXP_SHIFT) + (mantissa - IMPLICIT_BIT)
    } else {
        mantissa
    };

    // Adding the round bit to the packed form carries the significand into the
    // exponent, and `MAX` into `+∞`.
    f128::from_bits(packed + u128::from(up))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_and_special() {
        assert_eq!(expq(0.0).to_bits(), 1.0_f128.to_bits());
        assert_eq!(expq(-0.0).to_bits(), 1.0_f128.to_bits());
        assert_eq!(exp2q(10.0).to_bits(), 1024.0_f128.to_bits());
        assert_eq!(exp2q(-16382.0).to_bits(), f128::MIN_POSITIVE.to_bits());
        assert_eq!(exp2q(-16494.0).to_bits(), 1);
        assert_eq!(exp10q(3.0).to_bits(), 1000.0_f128.to_bits());
        assert_eq!(expq(f128::NEG_INFINITY).to_bits(), 0.0_f128.to_bits());
        assert!(expq(f128::INFINITY).is_infinite());
        assert!(expq(f128::NAN).is_nan());
        assert!(expq(f128::MAX).is_infinite());
        assert_eq!(expq(-f128::MAX).to_bits(), 0.0_f128.to_bits());
    }

    #[test]
    fn known_values() {
        // e and 1/e, correctly rounded.
        let e = f128::from_bits(0x4000_5bf0_a8b1_4576_9535_5fb8_ac40_4e7a);
        assert_eq!(expq(1.0).to_bits(), e.to_bits());
        assert_eq!(
            expq(-1.0).to_bits(),
            f128::from_bits(0x3ffd_78b5_6362_cef3_7c6a_eb7b_1e0a_4154).to_bits()
        );
        assert_eq!(exp2q(0.5).to_bits(), core::f128::consts::SQRT_2.to_bits());
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

    const PRECISION: u32 = 300;
    const SAMPLES: u64 = 200_000;

    fn mix(i: u64) -> u64 {
        let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A random significand and sign at an exponent uniform over `[-120, top]`,
    /// the whole span the reduction sees.
    fn sample(i: u64, top: i32) -> f128 {
        let bits = u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64;
        let exponent = (BIAS - 120) as u128 + (bits >> 120) % (top + 121) as u128;

        f128::from_bits(bits & SIGN_MASK | exponent << EXP_SHIFT | bits & MANTISSA_MASK)
    }

    /// The fast leg's raw `(exponent, significand)`, as [`exp_generic`] sees it.
    fn leg(x: f128, l: &Reduction) -> (i32, u128) {
        let (m, e) = split(x.to_bits() & !SIGN_MASK);
        let y = signed(reduce(m, e, l.head), x.is_sign_negative());

        fast(y[2] as i32, y[1])
    }

    /// Worst `|leg − f| / ZIV_GATE` over the sample, the error measured in units
    /// of the significand's last bit — exactly what the gate compares.
    fn worst_ratio(top: i32, l: &Reduction, f: impl Fn(&Float) -> Float) -> (f64, f128) {
        let mut worst = 0.0;
        let mut worst_x = 0.0;

        for i in 0..SAMPLES {
            let x = sample(i, top);
            let (n, r) = leg(x, l);
            let unit = Float::with_val(PRECISION, 2).pow(n - 127);
            let truth = f(&Float::with_val(PRECISION, x)) / unit;
            let ratio = Float::with_val(PRECISION, truth - Float::with_val(PRECISION, r))
                .abs()
                .to_f64()
                / ZIV_GATE as f64;

            if ratio > worst {
                worst = ratio;
                worst_x = x;
            }
        }
        (worst, worst_x)
    }

    fn certify(name: &str, top: i32, l: &Reduction, f: impl Fn(&Float) -> Float) {
        let (worst, x) = worst_ratio(top, l, f);
        println!("{name} fast leg: worst |err|/gate = {worst:.4} at x={x:?}");
        assert!(
            worst < 0.5,
            "{name} gate covers only {:.2}× the slip at x={x:?}",
            1.0 / worst
        );
    }

    #[test]
    fn expq_fast_leg_is_sound() {
        certify("expq", 13, &LOG2E, |x| x.clone().exp());
    }

    #[test]
    fn exp2q_fast_leg_is_sound() {
        certify("exp2q", 14, &ONE, |x| x.clone().exp2());
    }

    #[test]
    fn exp10q_fast_leg_is_sound() {
        certify("exp10q", 12, &LOG2_10, |x| x.clone().exp10());
    }
}
