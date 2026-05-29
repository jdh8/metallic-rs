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
