use core::ops::{Add, Div, Mul};

/// Fast C `ldexp` assuming normal argument and result
#[inline]
pub const fn fast_ldexp(x: f64, n: i64) -> f64 {
    const SHIFT: u32 = f64::MANTISSA_DIGITS - 1;
    f64::from_bits((x.to_bits() as i64 + (n << SHIFT)) as u64)
}

/// Represents `greater` + `lesser` where |`greater`| >> |`lesser`|
///
/// This structure serves as a high precision intermediate format consisting of
/// a pair of `f64`.  See [double-double arithmetic][dd] on Wikipedia for more
/// details.
///
/// [dd]:
///     https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format#Double-double_arithmetic
///
/// Sometimes, the sum is *normalized* when `f64`(`high` + `low`) = `high`. This
/// form preserves the most precision.
///
/// Arithmetic operations usually **breaks** normalization.  This type works as
/// an intermediate format.  Performance cost outweighs the precision gain from
/// renormalization.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DoubleDouble {
    /// The part with the larger absolute value
    pub high: f64,

    /// The part with the smaller absolute value
    pub low: f64,
}

/// Create a new `DoubleDouble` with the Fast2Sum algorithm
///
/// This algorithm produces normalized results when exponent(`a`) ≥
/// exponent(`b`).  It is sufficient when |`a`| ≥ |`b`|.
#[inline]
pub const fn fast_sum(a: f64, b: f64) -> DoubleDouble {
    let high = a + b;
    let low = a - high + b;
    DoubleDouble { high, low }
}

impl DoubleDouble {
    /// Create a normalized pair from an unordered sum
    #[inline]
    pub const fn from_sum(x: f64, y: f64) -> Self {
        let high = x + y;
        let diff = high - x;
        let low = y - diff + (diff - high + x);
        Self { high, low }
    }

    /// Create a normalized pair from a product
    #[inline]
    pub fn from_product(x: f64, y: f64) -> Self {
        let high = x * y;
        let low = crate::fma(x, y, -high);
        Self { high, low }
    }

    /// Create a normalized pair from a quotient
    #[inline]
    pub fn from_quotient(x: f64, y: f64) -> Self {
        let high = x / y;
        let low = crate::fma(high, -y, x) / y;
        Self { high, low }
    }

    /// The reciprocal `1 / self` as a double-double
    ///
    /// One Newton step `y·(2 − self·y)` from the `f64` seed `1/high` reaches about
    /// twice `f64` precision.
    #[inline]
    pub fn recip(self) -> Self {
        let y = 1.0 / self.high;
        (self * -y
            + Self {
                high: 2.0,
                low: 0.0,
            })
            * y
    }

    /// `self / other` to ≈2⁻¹⁰³ relative, for Ziv-gated fast paths
    ///
    /// One `f64` division: seed `y = 1/other.high`, take `h = self.high·y`,
    /// and fold the fused residual `r = self − h·other` (both words of each
    /// operand) into the low word `r·y`.  Versus `other.recip()` (whose Newton
    /// step is a serial double-double multiply, add, and multiply after the
    /// divide) followed by a double-double multiply, this halves the serial
    /// chain; the dropped second-order terms are ≈2⁻¹⁰³ relative — far below
    /// every fast-leg Ziv budget.  The accurate paths keep `recip`.
    #[inline]
    pub fn div_fast(self, other: Self) -> Self {
        let y = 1.0 / other.high;
        let h = self.high * y;
        let r = crate::fma(h, -other.low, crate::fma(h, -other.high, self.high)) + self.low;
        Self {
            high: h,
            low: r * y,
        }
    }

    /// `self + other` without renormalization, Fast2Sum on the high words
    ///
    /// Requires `exponent(self.high) ≥ exponent(other.high)` (e.g.
    /// `|self.high| ≥ |other.high|`).  The polynomial folds on Ziv-gated fast
    /// paths use this when their coefficient tables guarantee the ordering
    /// (each caller asserts so in a `fold_ordering` test): it skips the full
    /// `Add`'s 2Sum and renormalizing `fast_sum`, halving the fold's serial
    /// chain.  The result's `low` may grow to a few ulps of `high`; `Mul`,
    /// `Add`, and the Ziv gates' `high + (low ± eps)` all accept that form.
    #[inline]
    pub fn add_ordered(self, other: Self) -> Self {
        let s = fast_sum(self.high, other.high);
        Self {
            high: s.high,
            low: s.low + (self.low + other.low),
        }
    }

    /// `self + other` without renormalization, 2Sum on the high words (any
    /// magnitudes) — [`Self::add_ordered`] without the precondition.
    #[inline]
    pub fn add_loose(self, other: Self) -> Self {
        let s = Self::from_sum(self.high, other.high);
        Self {
            high: s.high,
            low: s.low + (self.low + other.low),
        }
    }
}

/// Fast multiplication that breaks normality
impl Mul for DoubleDouble {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        let product = Self::from_product(self.high, other.high);
        // Cross terms are compensation: use the true FMA so the low word stays
        // meaningful even on targets without hardware FMA (see `crate::fma`).
        let low = crate::fma(self.high, other.low, product.low);
        let low = crate::fma(self.low, other.high, low);

        Self {
            high: product.high,
            low,
        }
    }
}

/// Fast multiplication that breaks normality
impl Mul<f64> for DoubleDouble {
    type Output = Self;

    #[inline]
    fn mul(self, other: f64) -> Self {
        let product = Self::from_product(self.high, other);

        Self {
            high: product.high,
            // Compensation term: use the true FMA (see `crate::fma`).
            low: crate::fma(self.low, other, product.low),
        }
    }
}

/// Fast division that breaks normality
impl Div<f64> for DoubleDouble {
    type Output = Self;

    #[inline]
    fn div(self, other: f64) -> Self {
        let high = self.high / other;
        let low = (crate::fma(high, -other, self.high) + self.low) / other;
        Self { high, low }
    }
}

/// Double-double addition that breaks normality
impl Add for DoubleDouble {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        let sum = Self::from_sum(self.high, other.high);
        let low = sum.low + (self.low + other.low);
        fast_sum(sum.high, low)
    }
}

/// Round a normalized positive double-double to the nearest `f32`
///
/// A plain `value.high as f32` can double-round when `value.high` lands on an
/// `f32` midpoint: the cast rounds to even before the low word breaks the tie.
/// Rounding `value.high` to odd in `f64` first (in the direction of `value.low`)
/// sidesteps this — every `f32` midpoint has at least 28 trailing zero bits in
/// `f64`, hence is even, so the odd nudge lands on the correct side before the
/// final round to nearest.
#[inline]
pub fn round(value: DoubleDouble) -> f32 {
    let bits = value.high.to_bits();

    let odd = if value.low == 0.0 || bits & 1 == 1 {
        value.high
    } else if value.low > 0.0 {
        f64::from_bits(bits + 1)
    } else {
        f64::from_bits(bits - 1)
    };

    odd as f32
}

/// Correctly round `x + c`, where `x` is an **exact** `f64` and `c` a
/// double-double correction with `|c| ≤ |x|` (so `x` is the leading term).
///
/// This is the *result-anchored* finisher shared by the small-`|x|` legs of
/// `log1p`, `atanh`, and `asinh`: each computes only the tiny correction
/// `c = f(x) − x` to double-double precision (its relative error rides the
/// small `c`, not the result), then adds the exact `x` back here.
///
/// The two residuals combine exactly — `x + c.high = s` (2Sum), then
/// `s.low + c.low = w` (2Sum) with an exact sticky tail `we` — so `w` is
/// rounded to odd in the direction of `we` before the final round-to-nearest
/// add `s.high + w`.  Because `we` is the *exact* remainder, the round-to-odd
/// breaks every ½-ulp tie by the true sign of the sub-ulp tail (Boldo–Melquiond),
/// exactly as [`round`] does for the `f32` cast.
#[inline]
pub fn round_anchored(x: f64, c: DoubleDouble) -> f64 {
    let s = DoubleDouble::from_sum(x, c.high);
    let DoubleDouble { high: w, low: we } = DoubleDouble::from_sum(s.low, c.low);
    let w = if we == 0.0 || w.to_bits() & 1 == 1 {
        w
    } else if we.is_sign_positive() == w.is_sign_positive() {
        // Step `w` away from zero (toward `we`): magnitudes share a sign.
        f64::from_bits(w.to_bits() + 1)
    } else {
        f64::from_bits(w.to_bits() - 1)
    };
    s.high + w
}

/// Round a non-negative double-double to the nearest `f32`, safe across the
/// subnormal range and overflow
///
/// In the normal range this is [`round`] (round-to-odd then cast, which sends
/// overflow to `+∞`).  Below `f32::MIN_POSITIVE` a plain cast would round twice,
/// so the value is quantized once on the `2⁻¹⁴⁹` subnormal grid, exactly like
/// the subnormal branch of the `f64` exponential.
#[inline]
pub fn round_general(value: DoubleDouble) -> f32 {
    if value.high >= f64::from(f32::MIN_POSITIVE) {
        return round(value);
    }

    let high = value.high * crate::exp2i(149);
    let low = value.low * crate::exp2i(149);
    let n = high.round_ties_even();
    let n = n + ((high - n) + low).round_ties_even();

    (n * crate::exp2i(-149)) as f32
}

/// Square root of a non-negative double-double, refined by one Newton step
/// `h + (s − h²)/(2h)` to ≈2⁻¹⁰⁵ relative
///
/// `s.high` must be strictly positive.
#[inline]
pub fn sqrt_dd(s: DoubleDouble) -> DoubleDouble {
    let h = s.high.sqrt();
    let h2 = DoubleDouble::from_product(h, h);
    let residual = (s.high - h2.high) + (s.low - h2.low);
    fast_sum(h, residual * (0.5 / h))
}

/// Round a non-negative double-double `value · 2ⁿ` to the nearest `f64`, safe
/// across the subnormal range
///
/// `value` must be a normalized pair with `value.high ∈ [1, 2)`, so the result
/// is normal exactly when `n ≥ −1022` and the integer-grid shift below stays
/// within an exact `i64`.  The caller must keep the result finite (clamp
/// overflow before calling); gradual underflow into the subnormals is handled
/// here.  This is the `f64`-output analogue of [`round_general`] and the shared
/// reconstruction tail of the `f64` exponential family.
#[inline]
pub fn round_general64(value: DoubleDouble, n: i64) -> f64 {
    if n >= -1022 {
        // Normal result: scaling by 2ⁿ is exact, so one rounding of the pair.
        return fast_ldexp(value.high + value.low, n);
    }

    // Subnormal result: rounding the pair to `f64` and then scaling would round
    // twice.  Instead round the double-double on the integer grid at scale
    // 2⁻¹⁰⁷⁴ (the subnormal ulp): `m = (high + low) · 2^(n + 1074)` lies in
    // [0, 2⁵²], round it once to an integer, then `m · 2⁻¹⁰⁷⁴` is exact.
    let shift = n + 1074;
    let high = fast_ldexp(value.high, shift);
    let low = fast_ldexp(value.low, shift);

    // `high` may carry a half-integer resolution at this scale, so `high + low`
    // would discard the fine part of `low`.  Round `high`, then correct with the
    // exact residual `(high − n0) + low`.
    let n0 = high.round_ties_even();
    let n = n0 + ((high - n0) + low).round_ties_even();

    // 2⁻¹⁰⁷⁴ is the smallest positive subnormal, i.e. `f64::from_bits(1)`.
    n * f64::from_bits(1)
}

/// Round a signed double-double `value · 2ⁿ` to the nearest `f64`, safe across
/// the subnormal range, for an arbitrary-magnitude `value`
///
/// Unlike [`round_general64`] (which needs `value.high ∈ [1, 2)`), this
/// normalizes any nonzero-`high` `value` to a `[1, 2)` mantissa first, folds its
/// binary exponent into `n`, and restores the sign — the finisher for `tgamma`,
/// whose double-double result carries its own large dynamic range and sign.  The
/// caller must keep the result finite (clamp overflow beforehand); `value.high`
/// must be a nonzero normal `f64`.
#[inline]
pub fn round_general_signed64(value: DoubleDouble, n: i64) -> f64 {
    // Take the magnitude as a positive pair, then normalize its high word to [1, 2).
    let high = value.high.abs();
    let low = if value.high < 0.0 {
        -value.low
    } else {
        value.low
    };
    let e = (high.to_bits() >> (f64::MANTISSA_DIGITS - 1)) as i64 - 1023;
    let mantissa = DoubleDouble {
        high: fast_ldexp(high, -e),
        low: fast_ldexp(low, -e),
    };

    round_general64(mantissa, e + n).copysign(value.high)
}

/// Round a signed normal-range double-double to the nearest `f32`
///
/// Rounds the magnitude to odd then restores the sign, so [`round`]'s
/// positive-only round-to-odd applies on either side of zero.
#[inline]
pub fn round_signed(value: DoubleDouble) -> f32 {
    let magnitude = round(DoubleDouble {
        high: value.high.abs(),
        low: if value.high < 0.0 {
            -value.low
        } else {
            value.low
        },
    });

    magnitude.copysign(value.high as f32)
}
