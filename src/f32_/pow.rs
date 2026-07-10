use crate::f64_::double::{DoubleDouble, fast_ldexp, fast_sum, round_general};
use crate::f64_::pow::{EXP2_CE, EXP2_FAST, log2_dd, log2_fast_path, poly_dd};
use core::cmp::Ordering;
use core::num::FpCategory;

/// `log₂e` as a double-double, the base-2 lift of [`log2p1_dd`].
const LOG2E_DD: DoubleDouble = DoubleDouble {
    high: 1.442_695_040_888_963_4,
    low: 2.035_527_374_093_103_3e-17,
};

/// `2^e` correctly rounded to `f32`, taking a double-double exponent
///
/// Splits `e = n + h` with `n = round(e)` and `|h| ≤ ½`, evaluates `2ʰ` in
/// double-double via [`EXP2_CE`], scales by `2ⁿ` with [`fast_ldexp`], and rounds
/// the double-double with [`round_general`] (round-to-odd, subnormal-safe).  The
/// argument carries enough precision that the round is correct for [`powf`].
#[inline]
fn exp2_dd(e: DoubleDouble) -> f32 {
    if e.high > 130.0 {
        return f32::INFINITY;
    }
    if e.high < -160.0 {
        return 0.0;
    }

    let n = e.high.round_ties_even();
    let h = DoubleDouble::from_sum(e.high - n, e.low);
    let m = poly_dd(h, &EXP2_CE);

    // SAFETY: `-160 ≤ e.high ≤ 130` bounds `n`, so the scaling stays in range.
    let n = unsafe { n.to_int_unchecked() };

    round_general(DoubleDouble {
        high: fast_ldexp(m.high, n),
        low: fast_ldexp(m.low, n),
    })
}

/// `xʸ` for finite positive `x ≠ 1`, correctly rounded to `f32`
///
/// Fast path: `2^(y·log₂x)` entirely in `f64`.  The fast result is good to
/// ≈2⁻⁴⁵ relative, so unless its discarded low 29 bits land within `0x2000` of
/// the round-to-nearest midpoint — or it falls outside the normal `f32` range,
/// where the bit test does not apply — the single rounding is correct.  The few
/// ambiguous inputs (and the subnormal / near-overflow ends) take the
/// double-double [`log2_dd`]→`×y`→[`exp2_dd`] path.
#[inline]
fn powf_core(x: f64, y: f64) -> f32 {
    let e = log2_fast_path(x) * y;

    if e > 130.0 {
        return f32::INFINITY;
    }
    if e < -160.0 {
        return 0.0;
    }

    let n = e.round_ties_even();

    #[allow(clippy::cast_possible_truncation)]
    let r = fast_ldexp(crate::poly(e - n, &EXP2_FAST), n as i64);

    if r >= f64::from(f32::MIN_POSITIVE)
        && r < crate::exp2i(127)
        && (((r.to_bits() & 0x1FFF_FFFF) as i64) - 0x1000_0000).abs() > 0x2000
    {
        return r as f32;
    }

    exp2_dd(log2_dd(x) * y)
}

/// Raise to a floating-point power
#[must_use]
#[inline]
pub fn powf(x: f32, y: f32) -> f32 {
    #[inline]
    fn magnitude(x: f32, y: f32) -> f32 {
        match x.classify() {
            FpCategory::Nan => f32::NAN,
            FpCategory::Infinite => match y.partial_cmp(&0.0) {
                Some(Ordering::Greater) => f32::INFINITY,
                Some(Ordering::Less) => 0.0,
                Some(Ordering::Equal) => 1.0,
                None => f32::NAN,
            },
            FpCategory::Zero => match y.partial_cmp(&0.0) {
                Some(Ordering::Greater) => 0.0,
                Some(Ordering::Less) => f32::INFINITY,
                Some(Ordering::Equal) => 1.0,
                None => f32::NAN,
            },
            _ => match x {
                1.0 => 1.0,
                x if x.is_sign_negative() => f32::NAN,
                // xʸ = 2^(y·log₂x): a fast f64 pass with a Ziv gate, falling
                // back to double-double where the `×y` amplification of
                // `log₂x`'s error would otherwise cross an f32 rounding bound.
                _ => powf_core(x.into(), f64::from(y)),
            },
        }
    }

    #[inline]
    fn is_integer(x: f32) -> bool {
        x.trunc().eq(&x)
    }

    if y == 0.0 {
        return 1.0;
    }

    if x.is_sign_negative() && is_integer(y) {
        let sign = if is_integer(0.5 * y) { 1.0 } else { -1.0 };
        return sign * magnitude(-x, y);
    }

    magnitude(x, y)
}

/// `log₂(1 + x)` as a double-double, good to ≈2⁻⁸⁰ relative — the fallback
/// argument of [`compoundf`]
///
/// For `|x| < 2⁻²⁹` the series `log₂e·(x + x²·(−½ + x/3 − x²/4))`: the
/// f32-promoted `x` is exact, the tail is plain `f64` (`|tail| ≤ x²/2`, so
/// its 2⁻⁵² relative rounding sits below `2⁻⁸²·x`), and the truncated `x⁵/5`
/// is under `2⁻¹¹⁶` relative.  Otherwise `1 + x` is formed exactly — an f32
/// mantissa plus the unit bit spans at most 53 bits — and fed to the shared
/// [`log2_dd`]; the one case that drops a bit is `x ≥ 2⁵³`, where the
/// Fast2Sum residual `c = 1` returns as the single low-word term
/// `c/s·log₂e ≤ 2⁻⁵³·log₂e`.
fn log2p1_dd(x: f64) -> DoubleDouble {
    if x.abs() < crate::exp2i(-29) {
        let tail = (x * x) * crate::poly(x, &[-0.5, 1.0 / 3.0, -0.25]);
        return LOG2E_DD * fast_sum(x, tail);
    }
    let s = 1.0 + x;
    let c = if x <= 1.0 {
        x - (s - 1.0)
    } else {
        1.0 - (s - x)
    };
    let d = log2_dd(s);
    DoubleDouble {
        high: d.high,
        low: crate::fast_mul_add(c / s, core::f64::consts::LOG2_E, d.low),
    }
}

/// `(1+x)ʸ` for finite `x > −1`, `x ≠ 0`, finite `y ∉ {0, 1}` — [`powf_core`]
/// with the base-`1+x` front end
///
/// Fast path: `2^(y·log₂(1+x))` entirely in `f64` — the series leg below
/// `2⁻²⁹` (where `1 + x` would round), the exact `1 + x` through
/// [`log2_fast_path`] elsewhere.  The relative error stays within [`powf`]'s
/// ambiguity window (the `x ≥ 2⁵³` dropped unit contributes ≤ 2⁻⁵¹ to the
/// exponent), so the same discarded-bits test decides when the single
/// rounding is safe; the ambiguous inputs and the subnormal/overflow ends
/// take [`log2p1_dd`]`·y` → [`exp2_dd`].
fn compoundf_core(x: f64, y: f64) -> f32 {
    let l = if x.abs() < crate::exp2i(-29) {
        core::f64::consts::LOG2_E * crate::fast_mul_add(-0.5 * x, x, x)
    } else {
        log2_fast_path(1.0 + x)
    };
    let e = l * y;

    if e > 130.0 {
        return f32::INFINITY;
    }
    if e < -160.0 {
        return 0.0;
    }

    let n = e.round_ties_even();

    #[allow(clippy::cast_possible_truncation)]
    let r = fast_ldexp(crate::poly(e - n, &EXP2_FAST), n as i64);

    if r >= f64::from(f32::MIN_POSITIVE)
        && r < crate::exp2i(127)
        && (((r.to_bits() & 0x1FFF_FFFF) as i64) - 0x1000_0000).abs() > 0x2000
    {
        return r as f32;
    }

    if let Some(exact) = exact_compound(x, y) {
        return exact;
    }
    exp2_dd(log2p1_dd(x) * y)
}

/// Exact integer square root, or `None` when `m` is not a perfect square.
fn exact_sqrt(m: u64) -> Option<u64> {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let r = (m as f64).sqrt().round() as u64;
    (r.checked_mul(r) == Some(m)).then_some(r)
}

/// The exact-or-midpoint results the double-double fallback cannot decide
///
/// When `(1+x)ʸ` is a dyadic rational with at most 25 significant bits it can
/// sit exactly on an f32 value or midpoint, where the fallback's ≈2⁻⁸⁰ slack
/// still lands on a random side (round-to-nearest-even must decide from the
/// exact value: e.g. `compoundf(2⁴⁸+2²⁵, ½) = 2²⁴+1`, a midpoint that rounds
/// even to 2²⁴).  Such results only arise from an exactly-representable
/// `1 + x = m·2ᵉ` and `y = ±n·2ᵍ` with a tiny integer skeleton; resolve them
/// in exact integer arithmetic and round the exact `f64` once.  Everything
/// else returns `None`: a result with ≥ 26 significant bits keeps every f32
/// boundary at least ≈2⁻²⁶ away, far beyond the fallback's slack.
fn exact_compound(x: f64, y: f64) -> Option<f32> {
    let s = 1.0 + x;
    let c = if x <= 1.0 {
        x - (s - 1.0)
    } else {
        1.0 - (s - x)
    };
    if c != 0.0 {
        return None; // 1 + x inexact ⇒ the result is never a short dyadic
    }

    // s = m·2^e with m odd; s is a positive normal (x > −1, x ≠ 0).
    let sb = s.to_bits();
    let m = (sb & ((1_u64 << 52) - 1)) | (1 << 52);
    let mt = m.trailing_zeros();
    let m = m >> mt;
    let e = ((sb >> 52) as i64) - 1075 + i64::from(mt);

    // y = ±n·2^g with n odd; y is finite and nonzero.
    let yb = y.abs().to_bits();
    let n = (yb & ((1_u64 << 52) - 1)) | (1 << 52);
    let nt = n.trailing_zeros();
    let n = n >> nt;
    let g = ((yb >> 52) as i64) - 1075 + i64::from(nt);

    if m == 1 {
        // A power-of-two base: (2^e)^y = 2^(e·y), exact whenever e·y is an
        // integer (the f64 product of an 11-bit e and a 25-bit y is exact).
        #[allow(clippy::cast_possible_truncation)]
        let t = e as f64 * y;
        if t != t.trunc() || !(-160.0..=130.0).contains(&t) {
            return None; // an irrational 2^t never lands on a boundary
        }
        #[allow(clippy::cast_possible_truncation)]
        return Some(fast_ldexp(1.0, t as i64) as f32);
    }
    if y < 0.0 {
        return None; // 1/mⁿ with odd m ≥ 3 is never a dyadic rational
    }

    // Fractional y: m (and e) must survive |g| exact square roots.
    let (mut m, mut e) = (m, e);
    let mut g = g;
    while g < 0 {
        if e % 2 != 0 {
            return None;
        }
        m = exact_sqrt(m)?;
        e /= 2;
        g += 1;
    }

    // Integer exponent k = n·2^g: mᵏ must fit 25 bits for the result to
    // matter (m ≥ 3 ⇒ k ≤ 15), and the scaled exponent must stay in range.
    if g > 4 || n > 15 {
        return None;
    }
    let k = n << g;
    if k > 15 {
        return None;
    }
    let mut p: u64 = 1;
    for _ in 0..k {
        p = p.checked_mul(m)?;
        if p > 1 << 25 {
            return None;
        }
    }
    #[allow(clippy::cast_possible_wrap)]
    let ek = e.checked_mul(k as i64)?;
    if !(-160..=130).contains(&ek) {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    Some(fast_ldexp(p as f64, ek) as f32)
}

/// Signaling NaN: a NaN with the quiet bit clear.
#[inline]
fn is_snan(x: f32) -> bool {
    x.is_nan() && x.to_bits() & 0x0040_0000 == 0
}

/// Compound interest: `(1 + x)ʸ`, correctly rounded (C23's `compoundf`)
///
/// The special-value contract follows C23 F.10.4.1: `compoundf(±0, y) = 1`
/// for every `y` (even ±∞ and quiet NaN) and `compoundf(x, ±0) = 1` for
/// every `x ≥ −1` (and quiet-NaN `x`) — a signaling NaN still yields NaN;
/// `x < −1` (including −∞) is a domain error; `compoundf(−1, y)` is `+0` for
/// `y > 0` and `+∞` for `y < 0`.  The kernel is [`compoundf_core`]; `y = 1`
/// returns the exactly-representable `1 + x` directly.
#[must_use]
#[inline]
pub fn compoundf(x: f32, y: f32) -> f32 {
    if x == 0.0 {
        return if is_snan(y) { x + y } else { 1.0 };
    }
    if y == 0.0 {
        if is_snan(x) || x < -1.0 {
            return x + f32::NAN;
        }
        return 1.0; // includes x = +∞ and quiet NaN
    }
    if x.is_nan() || y.is_nan() {
        return x + y;
    }
    if x < -1.0 {
        return f32::NAN; // domain: 1 + x < 0, includes −∞
    }
    if y.is_infinite() {
        // (1+x) against 1 decides growth or decay; x = −1 decays too.
        return if (x > 0.0) == (y > 0.0) {
            f32::INFINITY
        } else {
            0.0
        };
    }
    if x == f32::INFINITY {
        return if y > 0.0 { f32::INFINITY } else { 0.0 };
    }
    if x == -1.0 {
        return if y > 0.0 { 0.0 } else { f32::INFINITY };
    }
    if y == 1.0 {
        // Exact: 1 + x in f64 is either exact (|x| ≥ 2⁻⁵²: the span fits) or
        // rounds within (1, 1 + 2⁻⁵²), whose f32 rounding is 1.0 either way.
        #[allow(clippy::cast_possible_truncation)]
        return (1.0 + f64::from(x)) as f32;
    }
    compoundf_core(x.into(), y.into())
}
