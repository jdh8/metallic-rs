use core::ops::Div;

/// Represents `greater` + `lesser` where |`greater`| ≥ |`lesser`|
///
/// The loose constraint makes arithmetic operations for [`DoubleDouble`]
/// faster.  However, this type does not check the ordering of magnitudes.
/// The order may become invalid after a series of operations.
pub struct OrderedSum {
    /// The part with the larger absolute value
    pub big: f64,

    /// The part with the smaller absolute value
    pub small: f64,
}

impl OrderedSum {
    /// Compute ordered sum
    ///
    /// This function returns the correct result when |`big`| ≥ |`small.big`|.
    /// The result is not guaranteed to be correct when |`big`| < |`small.big`|.
    #[inline]
    pub const fn ordered_add(big: f64, small: Self) -> Self {
        let sum = DoubleDouble::from_ordered_sum(Self {
            big,
            small: small.big,
        });
        Self {
            big: sum.high,
            small: small.small + sum.low,
        }
    }
}

/// High precision intermediate format consisting of a pair of `f64`
///
/// In this data structure, `high` is significantly larger than `low`, i.e.
/// `f64`(`high`) + `f64`(`low`) = `f64`(`high + low`).
///
/// See [double-double arithmetic][dd] on Wikipedia for more details.
///
/// [dd]: https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format#Double-double_arithmetic
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DoubleDouble {
    /// The high part, which dominates the sum
    high: f64,

    /// The low part, which is negligible when converted to `f64`
    low: f64,
}

impl DoubleDouble {
    /// Create a new pair from an ordered sum
    #[inline]
    pub const fn from_ordered_sum(pair: OrderedSum) -> Self {
        let high = pair.big + pair.small;
        let low = pair.big - high + pair.small;
        Self { high, low }
    }

    /// Create a new pair from an unordered sum
    #[inline]
    pub const fn from_sum(x: f64, y: f64) -> Self {
        let high = x + y;
        let diff = high - x;
        let low = y - diff + (diff - high + x);
        Self { high, low }
    }

    /// Create a new pair from a quotient
    #[inline]
    pub fn from_quotient(x: f64, y: f64) -> Self {
        let high = x / y;
        let low = high.mul_add(-y, x) / y;
        Self { high, low }
    }
}

impl From<DoubleDouble> for OrderedSum {
    #[inline]
    fn from(pair: DoubleDouble) -> Self {
        Self {
            big: pair.high,
            small: pair.low,
        }
    }
}

impl From<OrderedSum> for DoubleDouble {
    #[inline]
    fn from(pair: OrderedSum) -> Self {
        Self::from_ordered_sum(pair)
    }
}

impl Div<f64> for OrderedSum {
    type Output = Self;

    #[inline]
    fn div(self, other: f64) -> Self {
        let big = self.big / other;
        let small = (big.mul_add(-other, self.big) + self.small) / other;
        Self { big, small }
    }
}
