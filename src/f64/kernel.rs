use core::ops::Div;

/// Represents `greater` + `lesser` where |`greater`| >> |`lesser`|
///
/// This structure serves as a high precision intermediate format consisting of
/// a pair of `f64`.  See [double-double arithmetic][dd] on Wikipedia for more
/// details.
///
/// [dd]: https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format#Double-double_arithmetic
///
/// Sometimes, the sum is *normalized* when `f64`(`high` + `low`) = `high`.
/// This form preserves the most precision.
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

    /// Create a normalized pair from a quotient
    #[inline]
    pub fn from_quotient(x: f64, y: f64) -> Self {
        let high = x / y;
        let low = high.mul_add(-y, x) / y;
        Self { high, low }
    }
}

/// Fast division that does not preserve normality
impl Div<f64> for Sum {
    type Output = Self;

    #[inline]
    fn div(self, other: f64) -> Self {
        let high = self.high / other;
        let low = (high.mul_add(-other, self.high) + self.low) / other;
        Self { high, low }
    }
}
