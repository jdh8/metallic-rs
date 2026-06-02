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
pub struct Sum {
    /// The part with the larger absolute value
    pub high: f64,

    /// The part with the smaller absolute value
    pub low: f64,
}

/// Create a new `Sum` with the Fast2Sum algorithm
///
/// This algorithm produces normalized results when exponent(`a`) ≥
/// exponent(`b`).  It is sufficient when |`a`| ≥ |`b`|.
#[inline]
pub const fn fast_sum(a: f64, b: f64) -> Sum {
    let high = a + b;
    let low = a - high + b;
    Sum { high, low }
}

impl Sum {
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
        let low = x.mul_add(y, -high);
        Self { high, low }
    }

    /// Create a normalized pair from a quotient
    #[inline]
    pub fn from_quotient(x: f64, y: f64) -> Self {
        let high = x / y;
        let low = high.mul_add(-y, x) / y;
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
}

/// Fast multiplication that breaks normality
impl Mul for Sum {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        let product = Self::from_product(self.high, other.high);
        // Cross terms are compensation: use the true FMA so the low word stays
        // meaningful even on targets without hardware FMA (see `crate::mul_add`).
        let low = self.high.mul_add(other.low, product.low);
        let low = self.low.mul_add(other.high, low);

        Self {
            high: product.high,
            low,
        }
    }
}

/// Fast multiplication that breaks normality
impl Mul<f64> for Sum {
    type Output = Self;

    #[inline]
    fn mul(self, other: f64) -> Self {
        let product = Self::from_product(self.high, other);

        Self {
            high: product.high,
            // Compensation term: use the true FMA (see `crate::mul_add`).
            low: self.low.mul_add(other, product.low),
        }
    }
}

/// Fast division that breaks normality
impl Div<f64> for Sum {
    type Output = Self;

    #[inline]
    fn div(self, other: f64) -> Self {
        let high = self.high / other;
        let low = (high.mul_add(-other, self.high) + self.low) / other;
        Self { high, low }
    }
}

/// Double-double addition that breaks normality
impl Add for Sum {
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
pub fn round(value: Sum) -> f32 {
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

/// Round a non-negative double-double to the nearest `f32`, safe across the
/// subnormal range and overflow
///
/// In the normal range this is [`round`] (round-to-odd then cast, which sends
/// overflow to `+∞`).  Below `f32::MIN_POSITIVE` a plain cast would round twice,
/// so the value is quantized once on the `2⁻¹⁴⁹` subnormal grid, exactly like
/// the subnormal branch of the `f64` exponential.
#[inline]
pub fn round_general(value: Sum) -> f32 {
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
pub fn sqrt_dd(s: Sum) -> Sum {
    let h = s.high.sqrt();
    let h2 = Sum::from_product(h, h);
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
pub fn round_general64(value: Sum, n: i64) -> f64 {
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

/// Round a signed normal-range double-double to the nearest `f32`
///
/// Rounds the magnitude to odd then restores the sign, so [`round`]'s
/// positive-only round-to-odd applies on either side of zero.
#[inline]
pub fn round_signed(value: Sum) -> f32 {
    let magnitude = round(Sum {
        high: value.high.abs(),
        low: if value.high < 0.0 {
            -value.low
        } else {
            value.low
        },
    });

    magnitude.copysign(value.high as f32)
}
