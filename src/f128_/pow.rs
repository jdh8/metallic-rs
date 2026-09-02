//! The binary128 power function, `x^y = 2^(y·log2 x)`, on the engines of
//! [`log2q`](super::log2q) and [`exp2q`](super::exp2q).
//!
//! Three tiers, each deciding what the one before could not:
//!
//! 1. **Fast.** `log2 x` from the logarithm's 256-bit fast leg — its frame at
//!    2^-214 with an *absolute* slip under 2^-140 — times the exact significand
//!    of `y`, cut into the exponential's `(⌊y·log2 x⌋, fraction)` frame at
//!    2^-128, then the exponential's 128-bit fast leg.  The slip of the
//!    logarithm is magnified by `|y|`, so the Ziv gate widens with it:
//!    [`ZIV_GATE`](super::exp::ZIV_GATE) plus `|y|·2^-11`, which costs nothing
//!    below `|y| = 2^11` and hands everything above `|y| = 2^25` over outright.
//! 2. **Accurate.** The logarithm's 384-bit leg in *floating* form (its
//!    polynomial's `|log2(1 + z)|` at its own scale where the table terms
//!    cancel, so the relative accuracy stays 2^-251 however close `x` sits to
//!    1), the exact product with `y` cut into the 256-bit frame, and the
//!    exponential's 256-bit leg.  The relative error is under 2^-236 even at
//!    `|y·log2 x| = 2^14`, and [`ACCURATE_GATE`] refuses the 2^-122 of inputs
//!    within 2^-234 of a rounding tie.
//! 3. **Exact, then wide.**  Whatever the accurate leg refuses is first tried
//!    as an *exact* case: `x^y` is a dyadic rational — representable, or an
//!    exact midpoint no precision can resolve — only for a power-of-two base
//!    with an integer `E·y`, an integer `y ≤ 71` with `M^y < 2^114`, or
//!    `y = N/2^k` (`k ≤ 6`) with the odd part of `x` a perfect 2^k-th power
//!    ([`exact`] enumerates the family and rounds its integer value directly).
//!    Everything else is transcendental-hard only in the Ziv sense, and a
//!    table-free 640-bit tier decides it: `2^f` as `(e^(f·ln2/2^16))^(2^16)`
//!    by Taylor series and sixteen squarings, `log2 x` as one Newton step on
//!    the accurate leg's value through that same exponential.  Its relative
//!    error is under 2^-490, so by Ziv's heuristic the expected number of
//!    misrounded inputs over the whole 2^254-pair domain is about 2^-123.
//!
//! Everything is unsigned fixed point in integer limbs, like the engines it
//! rides: an `f128` multiply is soft-float on every target without hardware
//! binary128.

use super::exp::{self, GUARD, GUARD_HALF, GUARD_MASK};
use super::log::{self, Binary};
use super::pow_tables::{LN2, LOG2E};
use super::uint::{
    any_below, leading_zeros_256, shl_256, shr_sat, sub_256, wmul, wmul_128x256, wmul_128x384,
};
use super::{BIAS, EXP_MASK, EXP_SHIFT, IMPLICIT_BIT, MANTISSA_MASK, QUIET_BIT, SIGN_MASK, split};

/// The bit pattern of 1.
const ONE: u128 = (BIAS as u128) << EXP_SHIFT;

/// Half-width of the rounding-tie window the accurate leg refuses to decide,
/// in units of its significand's 2^-255.
///
/// `y·log2 x` carries the logarithm's relative slip, under 2^-251, into an
/// absolute one of at most `2^14·2^-251`, which the exponential turns into
/// `ln 2·2^-237` of the result — 2^17.5 units — on top of its own leg's few
/// dozen; 2^20 keeps the margin certified in [`ziv_soundness`].
const ACCURATE_GATE: u128 = 1 << 20;

/// `x` raised to the power `y`.
///
/// Follows C's `pow`: `x^±0 = 1` and `1^y = 1` for every `x` and `y`, NaN
/// included; a negative base needs an integer exponent, whose parity sets the
/// sign; `±0` and `±∞` bases follow the sign rules of Annex F.
#[must_use]
pub fn powq(x: f128, y: f128) -> f128 {
    let xb = x.to_bits();
    let yb = y.to_bits();
    let ax = xb & !SIGN_MASK;
    let ay = yb & !SIGN_MASK;

    if ay == 0 || xb == ONE {
        return 1.0;
    }
    // Zero, infinite or NaN base, or a nonfinite exponent.
    if ax.wrapping_sub(1) >= EXP_MASK - 1 || ay >= EXP_MASK {
        return special(xb, yb);
    }
    let sign = if xb >> 127 == 0 {
        0
    } else {
        match parity(ay) {
            Some(odd) => u128::from(odd) << 127,
            None => return f128::NAN,
        }
    };
    let (m, e) = split(ax);
    let (my, ey) = split(ay);
    let negative_y = yb >> 127 != 0;
    let (j, d, l, negative_l) = log2_fast(m, e);

    // Only `|x| = 1` leaves no logarithm at all: `(−1)^y = ±1`.
    if l == [0; 2] {
        return with_sign(1.0, sign);
    }
    let negative = negative_y ^ negative_l;

    // `|y·log2 x|` lies in `[2^(top − 2), 2^top)`: beyond 2^15 the result
    // saturates either way, below 2^-120 it rounds to 1.
    let top = ey + 43 - leading_zeros_256(l) as i32;

    if top >= 17 {
        return with_sign(if negative { 0.0 } else { f128::INFINITY }, sign);
    }
    if top <= -120 {
        return with_sign(1.0, sign);
    }
    let (n, r) = fast(l, my, ey, negative);
    let gate = exp::ZIV_GATE + extra(my, ey);

    if (n - (f128::MIN_EXP - 1)) as u32 > (f128::MAX_EXP - f128::MIN_EXP) as u32 {
        if exp::undecided(n, r, gate) {
            return with_sign(accurate(e, j, d, my, ey, negative_y, ax, ay), sign);
        }
        return with_sign(exp::round(n, r, 0), sign);
    }
    let rest = r & GUARD_MASK;

    if rest.abs_diff(GUARD_HALF) <= gate {
        return with_sign(accurate(e, j, d, my, ey, negative_y, ax, ay), sign);
    }
    // The gate has already ruled out an exact tie, so the round bit decides.
    with_sign(
        f128::from_bits(
            (((n + BIAS) as u128) << EXP_SHIFT)
                + ((r >> GUARD) - IMPLICIT_BIT)
                + u128::from(rest > GUARD_HALF),
        ),
        sign,
    )
}

/// `log2 |x|` in the logarithm's fast frame, 2^-214, from the exact reduction
/// `1 + z = m·2^(-j/2^18)` at 2^333: the estimate `j`, the reduction `d`, and
/// the frame value's magnitude and sign.
#[inline]
fn log2_fast(m: u128, e: i32) -> (u32, [u128; 3], [u128; 2], bool) {
    let j = log::crude_log2(m);
    let (high, low) = wmul(m, log::reciprocal(j));
    let d = [0, low, high.wrapping_sub(1 << 77)];
    let s = log::fast::<Binary>(e, j, log::z_fast(d));
    let negative = s[1] >> 127 != 0;

    (j, d, log::negate_if(s, negative), negative)
}

/// The fast leg: the exact product `my·|log2 x|·2^214` lands in the
/// exponential's frame — a 128-bit fraction under a 16-bit integer part —
/// after one right shift, whose dropped bits decide how the two's complement
/// of a negative exponent floors; then the exponential's 128-bit leg.
#[inline]
fn fast(l: [u128; 2], my: u128, ey: i32, negative: bool) -> (i32, u128) {
    let p = wmul_128x256(my, l);
    let shift = (198 - ey) as u32;
    let w = shr_sat(p, shift);
    let (f, n) = frame(w[0], w[1], negative, any_below(p, shift));

    exp::fast(n, f)
}

/// The widening of the fast leg's gate for `|y| = my·2^(ey − 112)`:
/// `⌈|y|·2^-11⌉`, saturated once it fills the fifteen-bit field.
#[inline]
fn extra(my: u128, ey: i32) -> u128 {
    if ey >= 25 {
        return 1 << 14;
    }
    my.checked_shr((123 - ey) as u32).unwrap_or(0) + 1
}

/// `(fraction, ⌊y⌋)` of the frame window `int:frac` of `|y|`, negated in two's
/// complement when `y < 0`: `−(W + d)` for a dropped tail `0 < d < 1` is `¬W`
/// in the window, `−W` when nothing was dropped.  The sign of `y·log2 x` is a
/// coin flip on random arguments, so it stays mask arithmetic.
#[inline]
fn frame(frac: u128, int: u128, negative: bool, dropped: bool) -> (u128, i32) {
    let mask = 0_u128.wrapping_sub(u128::from(negative));
    let (frac, carry) = (frac ^ mask).overflowing_add(mask & u128::from(!dropped));

    (frac, (int ^ mask).wrapping_add(u128::from(carry)) as i32)
}

/// Attach the sign of a negative base's odd power.
#[inline]
fn with_sign(magnitude: f128, sign: u128) -> f128 {
    f128::from_bits(magnitude.to_bits() | sign)
}

/// Whether the finite nonzero `|y|` is an integer, and if so whether it is odd.
#[inline]
fn parity(ay: u128) -> Option<bool> {
    let e = (ay >> EXP_SHIFT) as i32 - BIAS;

    if e < 0 {
        return None;
    }
    if e >= 113 {
        return Some(false);
    }
    let m = ay & MANTISSA_MASK | IMPLICIT_BIT;
    let below = (112 - e) as u32;

    if m & ((1 << below) - 1) != 0 {
        return None;
    }
    Some((m >> below) & 1 != 0)
}

/// The arguments with no logarithm to take: a zero, infinite or NaN base, or
/// an infinite or NaN exponent — `x^±0` and `1^y` are already gone.
#[cold]
#[inline(never)]
fn special(xb: u128, yb: u128) -> f128 {
    let ax = xb & !SIGN_MASK;
    let ay = yb & !SIGN_MASK;

    if ax > EXP_MASK || ay > EXP_MASK {
        return f128::from_bits(if ax > EXP_MASK { xb } else { yb } | QUIET_BIT);
    }
    let negative_y = yb >> 127 != 0;

    if ay == EXP_MASK {
        // `(−1)^±∞ = 1`; otherwise the side of 1 and the sign of ∞ decide.
        if ax == ONE {
            return 1.0;
        }
        return if (ax < ONE) == negative_y {
            f128::INFINITY
        } else {
            0.0
        };
    }
    // A zero or infinite base: an odd integer exponent keeps its sign.
    let sign = match parity(ay) {
        Some(true) => xb & SIGN_MASK,
        _ => 0,
    };
    let infinite = (ax == 0) == negative_y;

    f128::from_bits(sign | if infinite { EXP_MASK } else { 0 })
}

/// The accurate leg under its gate, and what it refuses handed on.
#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn accurate(
    e: i32,
    j: u32,
    d: [u128; 3],
    my: u128,
    ey: i32,
    negative_y: bool,
    ax: u128,
    ay: u128,
) -> f128 {
    let (n, r, wide_log) = accurate_leg(e, j, d, my, ey, negative_y);

    if undecided(n, r) {
        return exact_or_wide(ax, ay, negative_y, my, ey, wide_log);
    }
    exp::round(n, r[1], r[0])
}

/// The accurate leg: the logarithm at 384 bits in floating form, its exact
/// product with `y` in the 256-bit frame, and the exponential at 256 bits, as
/// `(exponent bump, significand scaled by 2^255)` — plus the logarithm, for
/// the wide tier to refine.
fn accurate_leg(
    e: i32,
    j: u32,
    d: [u128; 3],
    my: u128,
    ey: i32,
    negative_y: bool,
) -> (i32, [u128; 2], (bool, [u128; 3], i32)) {
    let wide_log = log::wide::<Binary>(e, j, d);
    let (negative_l, mag, k) = wide_log;
    let negative = negative_y ^ negative_l;

    // `|y·log2 x| = my·mag·2^(ey + k − 495)`, under 2^16 and over 2^-122, so
    // the 256-bit fraction starts between bits 224 and 362 of the product.
    let p = wmul_128x384(my, mag);
    let shift = (239 - ey - k) as u32;
    let w = shr_sat(p, shift);
    let dropped = any_below(p, shift);
    let y = negate_frame([w[0], w[1], w[2]], negative, dropped);
    let (n, r) = exp::exp2_frame(y);

    (n, r, wide_log)
}

/// [`frame`] on the 256-bit fraction and its integer limb.
#[inline]
fn negate_frame(w: [u128; 3], negative: bool, dropped: bool) -> [u128; 3] {
    if !negative {
        return w;
    }
    let (low, carry) = (!w[0]).overflowing_add(u128::from(!dropped));
    let (middle, carry) = (!w[1]).overflowing_add(u128::from(carry));

    [low, middle, (!w[2]).wrapping_add(u128::from(carry))]
}

/// Whether the accurate leg's 256-bit significand sits within
/// [`ACCURATE_GATE`] of the rounding tie of the bits it discards — 143 for a
/// normal result, more down the subnormal ladder.
fn undecided(n: i32, r: [u128; 2]) -> bool {
    let shift = 143 + (f128::MIN_EXP - 1 - n).max(0) as u32;

    // Overflow keeps no significand bit; below half the least subnormal every
    // significand rounds to zero.
    if n > f128::MAX_EXP - 1 || shift > 256 {
        return false;
    }
    let field = match shift {
        256 => r,
        128.. => [r[0], r[1] & (u128::MAX >> (256 - shift))],
        _ => [r[0] & (u128::MAX >> (128 - shift)), 0],
    };
    let d = sub_256(field, shl_256([1, 0], shift - 1));
    let m = if d[1] >> 127 == 0 {
        d
    } else {
        sub_256([0, 0], d)
    };

    m[1] == 0 && m[0] <= ACCURATE_GATE
}

/// The last resort: the exact family, then the 640-bit tier.
#[cold]
#[inline(never)]
fn exact_or_wide(
    ax: u128,
    ay: u128,
    negative_y: bool,
    my: u128,
    ey: i32,
    wide_log: (bool, [u128; 3], i32),
) -> f128 {
    if let Some(value) = exact(ax, ay, negative_y) {
        return value;
    }
    let (m, e) = split(ax);

    wide(m, e, my, ey, negative_y, wide_log)
}

// ===========================================================================
// The exact family: `x^y` a dyadic rational.
// ===========================================================================

/// The dyadic-rational powers, correctly rounded from their integer value.
///
/// With `|x| = M·2^E` and `|y| = N·2^F`, `M` and `N` odd, `x^y` is a dyadic
/// rational only when
///
/// - `M = 1`: `2^(E·y)`, exact whenever `E·y` is an integer;
/// - `y` a positive integer with `M^y < 2^114` (so `y ≤ 71`, as `3^72` has 115
///   bits);
/// - `y = N/2^k` with `k ≤ 6`, `2^k | E` and `M = m^(2^k)`, giving `m^N·2^(EN/2^k)`
///   with the same 114-bit bound on `m^N` (`3^64 < 2^113 < 5^64` caps `k`).
///
/// An odd integer of exactly 114 bits is an exact midpoint of binary128; one
/// of fewer bits is representable, unless it lands in the subnormal range,
/// where the fixed quantum re-decides — the integer rounder handles all
/// three.  Negative non-integer exponents and odd `M > 1` under a negative
/// integer exponent give denominators that never divide out.
fn exact(ax: u128, ay: u128, negative_y: bool) -> Option<f128> {
    let (m, e) = split(ax);
    let t = m.trailing_zeros();
    let (m, e) = (m >> t, e - 112 + t as i32);
    let (n, f) = split(ay);
    let t = n.trailing_zeros();
    let (n, f) = (n >> t, f - 112 + t as i32);

    if m == 1 {
        // `E ≠ 0` since `|x| ≠ 1`; `E·y` is an integer when `y` is, or when
        // `2^-F` divides `E`.
        let magnitude = if f >= 0 {
            // Beyond 2^21 the exponent saturates either way; below, the
            // product fits, and a saturated shift is far past the range too.
            if f > 21 {
                return Some(power_of_two(u128::MAX, (e < 0) != negative_y));
            }
            (u128::from(e.unsigned_abs()) * n).saturating_mul(1 << f)
        } else {
            if e.trailing_zeros() < (-f) as u32 {
                return None;
            }
            u128::from(e.unsigned_abs() >> -f) * n
        };
        return Some(power_of_two(magnitude, (e < 0) != negative_y));
    }
    if negative_y || n > 71 {
        return None;
    }
    let n = n as u32;

    if f >= 0 {
        if f > 6 || n << f > 71 {
            return None;
        }
        let y = n << f;
        return Some(scaled(checked_pow(m, y)?, i64::from(e) * i64::from(y)));
    }
    let k = (-f) as u32;

    if k > 6 || e & ((1 << k) - 1) != 0 {
        return None;
    }
    // `M = root^(2^k)`: `k` exact integer square roots.
    let mut root = m;

    for _ in 0..k {
        let r = root.isqrt();
        if r * r != root {
            return None;
        }
        root = r;
    }
    Some(scaled(
        checked_pow(root, n)?,
        i64::from(e >> k) * i64::from(n),
    ))
}

/// `base^exp` below 2^114, or `None`.
fn checked_pow(base: u128, mut exp: u32) -> Option<u128> {
    let mut result = 1_u128;
    let mut square = base;

    loop {
        if exp & 1 != 0 {
            result = result.checked_mul(square)?;
            if result >= 1 << 114 {
                return None;
            }
        }
        exp >>= 1;
        if exp == 0 {
            return Some(result);
        }
        // A square that overflows would have made the result overflow too.
        square = square.checked_mul(square)?;
    }
}

/// `k·2^g` correctly rounded, for `k < 2^114`.
fn scaled(k: u128, g: i64) -> f128 {
    let width = 128 - k.leading_zeros();
    let n = g + i64::from(width) - 1;

    exp::round(n.clamp(-1 << 20, 1 << 20) as i32, k << (128 - width), 0)
}

/// `2^±g` correctly rounded, saturating far outside the exponent range.
fn power_of_two(g: u128, negative: bool) -> f128 {
    let g = g.min(1 << 20) as i32;

    exp::round(if negative { -g } else { g }, 1 << 127, 0)
}

// ===========================================================================
// The 640-bit tier.
// ===========================================================================

/// Limbs of the wide tier's fixed-point values: 640 bits.
const LIMBS: usize = 10;

/// A 640-bit little-endian unsigned integer, or a two's complement one.
type Big = [u64; LIMBS];

/// `2^f` for the accurate leg's own refusals, at 640 bits.
///
/// `log2 x` is the accurate leg's value `L0` (relative error under 2^-248)
/// plus one Newton step through the exponential: `ε = x·2^(-L0) − 1` is
/// under 2^-234, and `log2(1 + ε) = log2 e·(ε − ε²/2 + ε³/3 − ⋯)` needs only
/// two terms at this width.  The exponential's relative error is under
/// 2^-617, so `log2 x` comes out within 2^-616 absolute and `y·log2 x`
/// within `|y|·2^-616 ≤ 2^-489`.
fn wide(
    m: u128,
    e: i32,
    my: u128,
    ey: i32,
    negative_y: bool,
    wide_log: (bool, [u128; 3], i32),
) -> f128 {
    let (n, r) = wide_raw(m, e, my, ey, negative_y, wide_log);
    let high = (u128::from(r[LIMBS - 1]) << 64) | u128::from(r[LIMBS - 2]);
    let sticky = u128::from(r[..LIMBS - 2].iter().any(|&limb| limb != 0));

    exp::round(n, high, sticky)
}

/// [`wide`] before its rounding: `(exponent bump, significand scaled by
/// 2^639)`.
fn wide_raw(
    m: u128,
    e: i32,
    my: u128,
    ey: i32,
    negative_y: bool,
    (negative_l, mag, k): (bool, [u128; 3], i32),
) -> (i32, Big) {
    // `|L0|·2^620 = mag·2^(k + 237)`, at most 636 bits.
    let l0 = shl(&from_384(mag), (k + 237) as u32);
    let l0 = if negative_l { neg(&l0) } else { l0 };

    // `−L0 = n1 + f1` with `n1 = ⌊−L0⌋` (twenty integer bits) and `f1 ∈ [0, 1)`.
    let minus = neg(&l0);
    let n1 = (minus[LIMBS - 1] as i64) >> 44;
    let f1 = shl(&minus, 20);
    let e1 = exp2_wide(&f1);

    // `x·2^(−L0) = m·e1·2^(e + n1 − 751)`, a hair from 1: its excess over 1,
    // at 2^-640, is the low 640 bits of the product placed at the point.
    let p = mul_u128(&e1, m);
    let s = 111 - e - n1 as i32;
    let q = shr(&p, s as u32);
    let eps: Big = q[..LIMBS].try_into().unwrap();
    let negative_eps = eps[LIMBS - 1] >> 63 != 0;
    let eps_magnitude = if negative_eps { neg(&eps) } else { eps };

    // `log2 e·ε` at 2^-639 and `log2 e·ε²` at 2^-639, brought to 2^-620 —
    // the latter halved on the way.
    let c1 = shr(&mul_hi(&eps_magnitude, &LOG2E), 19);
    let c2 = shr(&mul_hi(&mul_hi(&eps_magnitude, &eps_magnitude), &LOG2E), 20);
    let l = add(&l0, &if negative_eps { neg(&c1) } else { c1 });
    let l = sub(&l, &c2);
    let negative_l = l[LIMBS - 1] >> 63 != 0;
    let l = if negative_l { neg(&l) } else { l };
    let negative = negative_y ^ negative_l;

    // `|y·L|·2^640 = my·l·2^(ey − 92)`: under 2^656, so twelve limbs hold it.
    let p = mul_u128(&l, my);
    let (w, dropped) = if ey <= 92 {
        (
            shr(&p, (92 - ey) as u32),
            any_below_64(&p, (92 - ey) as u32),
        )
    } else {
        (shl(&p, (ey - 92) as u32), false)
    };
    let w = negate_wide(w, negative, dropped);

    (w[LIMBS] as i32, exp2_wide(&w[..LIMBS].try_into().unwrap()))
}

/// `2^f` for `f ∈ [0, 1)` at 2^-640, as a significand in `[2^639, 2^640)`:
/// `e^t` for `t = f·ln 2/2^16 < 2^-16.5` by Taylor series, squared sixteen
/// times.  Every truncation is a unit of 2^-639 or less and each squaring
/// doubles the relative error, so the result is within 2^-617 relative.
fn exp2_wide(f: &Big) -> Big {
    let t = shr(&mul_hi(f, &LN2), 16);
    let mut term = shr(&t, 1);
    let mut sum = add(&one(), &term);

    for k in 2.. {
        term = div_small(&mul_hi(&term, &t), k);
        if term == [0; LIMBS] {
            break;
        }
        sum = add(&sum, &term);
    }
    for _ in 0..16 {
        sum = shl(&mul_hi(&sum, &sum), 1);
    }
    sum
}

/// One at 2^-639.
const fn one() -> Big {
    let mut one = [0; LIMBS];
    one[LIMBS - 1] = 1 << 63;
    one
}

/// A 384-bit value in the low limbs of a [`Big`].
fn from_384(x: [u128; 3]) -> Big {
    let mut out = [0; LIMBS];

    for (i, limb) in x.iter().enumerate() {
        out[2 * i] = *limb as u64;
        out[2 * i + 1] = (*limb >> 64) as u64;
    }
    out
}

/// `a + b`, wrapping.
fn add<const N: usize>(a: &[u64; N], b: &[u64; N]) -> [u64; N] {
    let mut out = [0; N];
    let mut carry = 0;

    for i in 0..N {
        let sum = u128::from(a[i]) + u128::from(b[i]) + carry;
        out[i] = sum as u64;
        carry = sum >> 64;
    }
    out
}

/// `a − b`, wrapping.
fn sub<const N: usize>(a: &[u64; N], b: &[u64; N]) -> [u64; N] {
    let mut out = [0; N];
    let mut borrow = 0;

    for i in 0..N {
        let diff = (u128::from(a[i]) | 1 << 64) - u128::from(b[i]) - borrow;
        out[i] = diff as u64;
        borrow = u128::from(diff >> 64 == 0);
    }
    out
}

/// `−a` in two's complement.
fn neg<const N: usize>(a: &[u64; N]) -> [u64; N] {
    sub(&[0; N], a)
}

/// `a << shift`, for `shift < 64·N`.
fn shl<const N: usize>(a: &[u64; N], shift: u32) -> [u64; N] {
    let word = (shift / 64) as usize;
    let bits = shift % 64;
    let mut out = [0; N];

    for i in word..N {
        let carry = if bits == 0 || i == word {
            0
        } else {
            a[i - word - 1] >> (64 - bits)
        };
        out[i] = (a[i - word] << bits) | carry;
    }
    out
}

/// `a >> shift`, for `shift < 64·N`.
fn shr<const N: usize>(a: &[u64; N], shift: u32) -> [u64; N] {
    let word = (shift / 64) as usize;
    let bits = shift % 64;
    let mut out = [0; N];

    for i in word..N {
        let carry = match a.get(i + 1) {
            Some(&next) if bits != 0 => next << (64 - bits),
            _ => 0,
        };
        out[i - word] = (a[i] >> bits) | carry;
    }
    out
}

/// Whether any of the low `n` bits is set.
fn any_below_64<const N: usize>(a: &[u64; N], n: u32) -> bool {
    let word = (n / 64) as usize;
    let bits = n % 64;

    a[..word].iter().any(|&limb| limb != 0) || (bits != 0 && a[word] & ((1 << bits) - 1) != 0)
}

/// High 640 bits of a 640×640-bit product, truncated.
fn mul_hi(a: &Big, b: &Big) -> Big {
    let mut p = [0_u64; 2 * LIMBS];

    for i in 0..LIMBS {
        let mut carry = 0;

        for j in 0..LIMBS {
            let t = u128::from(a[i]) * u128::from(b[j]) + u128::from(p[i + j]) + carry;
            p[i + j] = t as u64;
            carry = t >> 64;
        }
        p[i + LIMBS] = carry as u64;
    }
    p[LIMBS..].try_into().unwrap()
}

/// The exact 768-bit product of a 640-bit value and a 128-bit one.
fn mul_u128(a: &Big, m: u128) -> [u64; LIMBS + 2] {
    let b = [m as u64, (m >> 64) as u64];
    let mut p = [0_u64; LIMBS + 2];

    for (i, &ai) in a.iter().enumerate() {
        let mut carry = 0;

        for (j, &bj) in b.iter().enumerate() {
            let t = u128::from(ai) * u128::from(bj) + u128::from(p[i + j]) + carry;
            p[i + j] = t as u64;
            carry = t >> 64;
        }
        p[i + 2] = carry as u64;
    }
    p
}

/// `a / d` for a small divisor.
fn div_small(a: &Big, d: u64) -> Big {
    let mut out = [0; LIMBS];
    let mut rest = 0_u128;

    for i in (0..LIMBS).rev() {
        let current = (rest << 64) | u128::from(a[i]);
        out[i] = (current / u128::from(d)) as u64;
        rest = current % u128::from(d);
    }
    out
}

/// [`frame`]'s negation on the wide frame `int:frac`.
fn negate_wide(w: [u64; LIMBS + 2], negative: bool, dropped: bool) -> [u64; LIMBS + 2] {
    if !negative {
        return w;
    }
    let mut out = [0; LIMBS + 2];
    let mut carry = u128::from(!dropped);

    for i in 0..LIMBS + 2 {
        let sum = u128::from(!w[i]) + carry;
        out[i] = sum as u64;
        carry = sum >> 64;
    }
    out
}

#[cfg(test)]
fn mix(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
fn mix128(i: u64) -> u128 {
    u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9_7F4A_7C15)) << 64
}

/// A pair with `|y·log2 x|` in about `[2^-118, 2^15)`: the base near 1 every
/// fourth draw, within a few binades of 1 every other fourth, anywhere
/// otherwise; the exponent of `y` uniform over the span that keeps the result
/// in range, either sign.  `log2 |log2 x|` comes in from the caller so the
/// sampler needs no MPFR.
#[cfg(test)]
fn sample(i: u64, log2_log2: impl Fn(f128) -> i32) -> (f128, f128) {
    let xb = mix128(2 * i);
    let yb = mix128(2 * i + 1);
    let x = match i % 4 {
        0 => {
            let k = (xb >> 112 & 0x7fff) % 112;
            let t = f128::from_bits((16383 - 112 + k) << 112 | xb & MANTISSA_MASK);
            if xb >> 127 == 0 { 1.0 + t } else { 1.0 - t }
        }
        1 => f128::from_bits(((xb >> 112) % 64 + 16383 - 32) << 112 | xb & MANTISSA_MASK),
        _ => f128::from_bits(((xb >> 112) % 0x7fff) << 112 | xb & MANTISSA_MASK | 1),
    };
    let ey = (-118 - log2_log2(x) + ((yb >> 112) % 132) as i32).clamp(-16382, 16383);
    let y =
        f128::from_bits(yb & SIGN_MASK | ((ey + BIAS) as u128) << EXP_SHIFT | yb & MANTISSA_MASK);

    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MPFR answers as bit patterns: `√10`, a base one ulp above 1 raised to
    /// `2^100` and one ulp below 1 to `−2^110` (the accurate leg's floating
    /// logarithm), `3^(1/3)`, `e^π`, the least normal to the `1/7`, `7^5000`
    /// near the top of the range, and `(1 + 2^-20)^(2^30)`.
    const KNOWN: [(u128, u128, u128); 8] = [
        (
            0x4002_4000_0000_0000_0000_0000_0000_0000,
            0x3ffe_0000_0000_0000_0000_0000_0000_0000,
            0x4000_94c5_83ad_a5b5_2920_4a2b_c830_cd9c,
        ),
        (
            0x3fff_0000_0000_0000_0000_0000_0000_0001,
            0x4063_0000_0000_0000_0000_0000_0000_0000,
            0x3fff_0010_0080_02aa_b555_7777_d27d_f7e1,
        ),
        (
            0x3ffe_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
            0xc06d_0000_0000_0000_0000_0000_0000_0000,
            0x3fff_2216_045b_6f5c_cf9c_ed68_8384_e06c,
        ),
        (
            0x4000_8000_0000_0000_0000_0000_0000_0000,
            0x3ffd_5555_5555_5555_5555_5555_5555_5555,
            0x3fff_7137_4491_23ef_65cd_de7f_16c5_6e32,
        ),
        (
            0x4000_5bf0_a8b1_4576_9535_5fb8_ac40_4e7a,
            0x4000_921f_b544_42d1_8469_898c_c517_01b8,
            0x4003_7240_46eb_0933_99ec_da74_89f9_ab74,
        ),
        (
            0x0001_0000_0000_0000_0000_0000_0000_0000,
            0x3ffc_2492_4924_9249_2492_4924_9249_2492,
            0x36da_a402_feeb_9c53_2ba6_200b_3c9d_800c,
        ),
        (
            0x4001_c000_0000_0000_0000_0000_0000_0000,
            0x400b_3880_0000_0000_0000_0000_0000_0000,
            0x76d3_b5f2_430e_9190_52d8_4345_5a3b_bd84,
        ),
        (
            0x3fff_0000_1000_0000_0000_0000_0000_0000,
            0x401d_0000_0000_0000_0000_0000_0000_0000,
            0x45c4_3f5a_e434_c81e_9e42_ce72_d862_10a4,
        ),
    ];

    #[test]
    fn special_values() {
        assert_eq!(powq(f128::NAN, 0.0).to_bits(), 1.0_f128.to_bits());
        assert_eq!(powq(1.0, f128::NAN).to_bits(), 1.0_f128.to_bits());
        assert_eq!(powq(-1.0, f128::INFINITY).to_bits(), 1.0_f128.to_bits());
        assert_eq!(powq(-1.0, f128::NEG_INFINITY).to_bits(), 1.0_f128.to_bits());
        assert!(powq(f128::NAN, 1.0).is_nan());
        assert!(powq(2.0, f128::NAN).is_nan());
        assert!(powq(-2.0, 0.5).is_nan());
        assert!(powq(-2.0, 1.5).is_nan());
        assert_eq!(powq(0.5, f128::INFINITY).to_bits(), 0.0_f128.to_bits());
        assert!(powq(0.5, f128::NEG_INFINITY).is_infinite());
        assert!(powq(2.0, f128::INFINITY).is_infinite());
        assert_eq!(powq(2.0, f128::NEG_INFINITY).to_bits(), 0.0_f128.to_bits());
        assert_eq!(powq(0.0, 3.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(powq(-0.0, 3.0).to_bits(), (-0.0_f128).to_bits());
        assert_eq!(powq(-0.0, 2.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(powq(-0.0, 0.5).to_bits(), 0.0_f128.to_bits());
        assert_eq!(powq(0.0, -3.0).to_bits(), f128::INFINITY.to_bits());
        assert_eq!(powq(-0.0, -3.0).to_bits(), f128::NEG_INFINITY.to_bits());
        assert_eq!(powq(-0.0, -2.0).to_bits(), f128::INFINITY.to_bits());
        assert_eq!(
            powq(0.0, f128::NEG_INFINITY).to_bits(),
            f128::INFINITY.to_bits()
        );
        assert_eq!(powq(f128::INFINITY, -1.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(
            powq(f128::INFINITY, 1.0).to_bits(),
            f128::INFINITY.to_bits()
        );
        assert_eq!(
            powq(f128::NEG_INFINITY, -3.0).to_bits(),
            (-0.0_f128).to_bits()
        );
        assert_eq!(powq(f128::NEG_INFINITY, -2.0).to_bits(), 0.0_f128.to_bits());
        assert_eq!(
            powq(f128::NEG_INFINITY, 3.0).to_bits(),
            f128::NEG_INFINITY.to_bits()
        );
        assert_eq!(
            powq(f128::NEG_INFINITY, 2.5).to_bits(),
            f128::INFINITY.to_bits()
        );
        assert_eq!(powq(-1.0, 3.0).to_bits(), (-1.0_f128).to_bits());
        assert_eq!(powq(-1.0, 1e30).to_bits(), 1.0_f128.to_bits());
        assert_eq!(powq(-1.0, -f128::MAX).to_bits(), 1.0_f128.to_bits());
    }

    #[test]
    fn exact_values() {
        assert_eq!(powq(2.0, 10.0).to_bits(), 1024.0_f128.to_bits());
        assert_eq!(powq(2.0, -10.0).to_bits(), (1.0_f128 / 1024.0).to_bits());
        assert_eq!(powq(-2.0, 3.0).to_bits(), (-8.0_f128).to_bits());
        assert_eq!(powq(-2.0, 4.0).to_bits(), 16.0_f128.to_bits());
        assert_eq!(powq(4.0, 0.5).to_bits(), 2.0_f128.to_bits());
        assert_eq!(powq(9.0, 0.5).to_bits(), 3.0_f128.to_bits());
        assert_eq!(powq(81.0, 0.25).to_bits(), 3.0_f128.to_bits());
        assert_eq!(powq(81.0, 0.75).to_bits(), 27.0_f128.to_bits());
        assert_eq!(powq(8.0, 1.0 / 3.0).to_bits(), 2.0_f128.to_bits());
        assert_eq!(powq(1.5, 2.0).to_bits(), 2.25_f128.to_bits());
        assert_eq!(
            powq(2.0, 0.5).to_bits(),
            core::f128::consts::SQRT_2.to_bits()
        );
        assert_eq!(
            powq(2.0, 16383.0).to_bits(),
            f128::from_bits(0x7ffe << 112).to_bits()
        );
        assert!(powq(2.0, 16384.0).is_infinite());
        assert_eq!(powq(2.0, -16494.0).to_bits(), 1);
        assert_eq!(powq(2.0, -16495.0).to_bits(), 0);
        assert_eq!(powq(0.5, 16495.0).to_bits(), 0);
        assert_eq!(powq(0.5, 16494.0).to_bits(), 1);
        // The least subnormal is `2^-16494`; its square root is `2^-8247`.
        assert_eq!(powq(f128::from_bits(1), 0.5).to_bits(), 8136 << 112);
        // `3^71` has 113 bits: representable.
        assert_eq!(
            powq(3.0, 71.0).to_bits(),
            (3_u128.pow(71) as f128).to_bits()
        );
        assert_eq!(
            powq(9.0, 35.5).to_bits(),
            (3_u128.pow(71) as f128).to_bits()
        );
        // `5^49` has 114 bits: an exact midpoint, rounded to even.
        let k = 5_u128.pow(49);
        let even = if (k >> 1) & 1 == 0 { k - 1 } else { k + 1 };
        assert_eq!(powq(5.0, 49.0).to_bits(), (even as f128).to_bits());
        assert_eq!(powq(25.0, 24.5).to_bits(), (even as f128).to_bits());
        assert_eq!(powq(625.0, 12.25).to_bits(), (even as f128).to_bits());
    }

    #[test]
    fn known_values() {
        for (x, y, want) in KNOWN {
            assert_eq!(
                powq(f128::from_bits(x), f128::from_bits(y)).to_bits(),
                want,
                "pow({x:#x}, {y:#x})"
            );
        }
    }

    /// The wide tier never runs on a random argument, so it is exercised
    /// directly: on the benchmark's band it must agree with the fast leg,
    /// whose gate certifies its answers.
    #[test]
    fn wide_tier_agrees() {
        for i in 0..2000_u64 {
            let (x, y) = sample(i, |x| (x.to_bits() >> 112) as i32 - BIAS);
            let ax = x.to_bits();
            let ay = y.to_bits() & !SIGN_MASK;
            let (m, e) = split(ax);
            let (my, ey) = split(ay);
            let (_, _, l, _) = log2_fast(m, e);
            let top = ey + 43 - leading_zeros_256(l) as i32;

            if !(-118..=14).contains(&top) {
                continue;
            }
            let (_, _, wide_log) = {
                let (j, d, _, _) = log2_fast(m, e);
                accurate_leg(e, j, d, my, ey, y < 0.0)
            };
            let got = wide(m, e, my, ey, y < 0.0, wide_log);
            assert_eq!(got.to_bits(), powq(x, y).to_bits(), "pow({x:?}, {y:?})");
        }
    }
}

/// MPFR certification that the gates cover their legs' true errors with the
/// 2× margin the project requires, and that the wide tier reaches the
/// precision its policy claims.  Run with
/// `CC=clang cargo +nightly test --release --features "f128 mpfr"`.
#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::*;
    use rug::{Float, ops::Pow};

    const PRECISION: u32 = 800;
    const SAMPLES: u64 = 100_000;

    /// `⌊log2 |log2 x|⌋`, for the sampler's choice of `y`.
    fn log2_log2(x: f128) -> i32 {
        Float::with_val(64, x).log2().abs().to_f64().log2().floor() as i32
    }

    /// `x^y` at 800 bits.
    fn truth(x: f128, y: f128) -> Float {
        Float::with_val(PRECISION, x).pow(Float::with_val(PRECISION, y))
    }

    /// The legs' inputs for a pair, or `None` where [`powq`] runs no leg at
    /// all: `|x| = 1`, or a saturating or unit result.
    fn setup(x: f128, y: f128) -> Option<(u128, i32, u32, [u128; 3], [u128; 2], bool, u128, i32)> {
        let (m, e) = split(x.to_bits());
        let (my, ey) = split(y.to_bits() & !SIGN_MASK);
        let (j, d, l, negative_l) = log2_fast(m, e);
        let top = ey + 43 - leading_zeros_256(l) as i32;

        if l == [0; 2] || top >= 17 || top <= -120 {
            return None;
        }
        Some((m, e, j, d, l, negative_l, my, ey))
    }

    #[test]
    fn fast_leg_is_sound() {
        let mut worst = 0.0;
        let mut at = (0.0, 0.0);

        for i in 0..SAMPLES {
            let (x, y) = sample(i, log2_log2);
            let Some((_, _, _, _, l, negative_l, my, ey)) = setup(x, y) else {
                continue;
            };
            // Above `|y| = 2^25` the gate fills the whole field and every
            // draw falls back, whatever the leg's error.
            if ey >= 25 {
                continue;
            }
            let (n, r) = fast(l, my, ey, (y < 0.0) ^ negative_l);
            let gate = exp::ZIV_GATE + extra(my, ey);
            let scaled = truth(x, y) / Float::with_val(PRECISION, 2).pow(n - 127);
            let ratio = Float::with_val(PRECISION, scaled - Float::with_val(PRECISION, r))
                .abs()
                .to_f64()
                / gate as f64;

            if ratio > worst {
                worst = ratio;
                at = (x, y);
            }
        }
        println!(
            "powq fast leg: worst |err|/gate = {worst:.4} at x={:?} y={:?}",
            at.0, at.1
        );
        assert!(
            worst < 0.5,
            "powq gate covers only {:.2}× the slip",
            1.0 / worst
        );
    }

    #[test]
    fn accurate_leg_is_sound() {
        let mut worst = 0.0;
        let mut at = (0.0, 0.0);

        for i in 0..SAMPLES / 4 {
            let (x, y) = sample(i, log2_log2);
            let Some((_, e, j, d, _, _, my, ey)) = setup(x, y) else {
                continue;
            };
            let (n, r, _) = accurate_leg(e, j, d, my, ey, y < 0.0);
            let scaled = truth(x, y) / Float::with_val(PRECISION, 2).pow(n - 255);
            let got = Float::with_val(PRECISION, r[1]) * Float::with_val(PRECISION, 2).pow(128)
                + Float::with_val(PRECISION, r[0]);
            let ratio =
                Float::with_val(PRECISION, scaled - got).abs().to_f64() / ACCURATE_GATE as f64;

            if ratio > worst {
                worst = ratio;
                at = (x, y);
            }
        }
        println!(
            "powq accurate leg: worst |err|/gate = {worst:.3e} at x={:?} y={:?}",
            at.0, at.1
        );
        assert!(
            worst < 0.5,
            "powq accurate gate covers only {:.2}× the slip",
            1.0 / worst
        );
    }

    /// The wide tier has no gate: its policy is a relative error under 2^-490,
    /// and its rounding must match MPFR's.
    #[test]
    fn wide_tier_is_accurate() {
        let mut worst: f64 = 0.0;
        let mut at = (0.0, 0.0);

        for i in 0..SAMPLES / 16 {
            let (x, y) = sample(i, log2_log2);
            let Some((m, e, j, d, _, _, my, ey)) = setup(x, y) else {
                continue;
            };
            let (_, _, wide_log) = accurate_leg(e, j, d, my, ey, y < 0.0);
            let (n, r) = wide_raw(m, e, my, ey, y < 0.0, wide_log);
            let scaled = truth(x, y) / Float::with_val(PRECISION, 2).pow(n - 639);
            let mut got = Float::with_val(PRECISION, 0);
            for (k, limb) in r.iter().enumerate() {
                got += Float::with_val(PRECISION, *limb)
                    * Float::with_val(PRECISION, 2).pow(64 * k as u32);
            }
            let relative =
                Float::with_val(PRECISION, scaled - got).abs().to_f64() / 2_f64.powi(639);
            if relative > worst {
                worst = relative;
                at = (x, y);
            }
            let want = super::super::mpfr::cr_binop(x, y, |a, b| {
                use rug::ops::PowAssignRound;
                a.pow_assign_round(b, rug::float::Round::Nearest)
            });
            assert_eq!(
                wide(m, e, my, ey, y < 0.0, wide_log).to_bits(),
                want.to_bits(),
                "pow({x:?}, {y:?})"
            );
        }
        println!(
            "powq wide tier: worst relative error = 2^{:.1} at x={:?} y={:?}",
            worst.log2(),
            at.0,
            at.1
        );
        assert!(worst < 2_f64.powi(-480));
    }
}
