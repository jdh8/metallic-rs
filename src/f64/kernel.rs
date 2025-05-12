use core::ops::{Add, Mul};

/// High precision intermediate format consisting of a pair of `f64`
///
/// <https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format#Double-double_arithmetic>
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Pair {
    /// High part
    high: f64,

    /// Low part
    low: f64,
}

impl Pair {
    /// Create a new pair from an ordered sum
    ///
    /// This function returns the correct result when |`x`| ≥ |`y`|.
    /// The result is not guaranteed to be correct when |`x`| < |`y`|.
    #[inline]
    const fn from_ordered_sum(greater: f64, lesser: f64) -> Self {
        let high = greater + lesser;
        let low = greater - high + lesser;
        Self { high, low }
    }

    /// Create a new pair from an unordered sum
    #[inline]
    const fn from_sum(x: f64, y: f64) -> Self {
        let high = x + y;
        let diff = high - x;
        let low = y - diff + (diff - high + x);
        Self { high, low }
    }

    /// Create a new pair from a product
    #[inline]
    fn from_product(x: f64, y: f64) -> Self {
        let high = x * y;
        let low = x.mul_add(y, -high);
        Self { high, low }
    }

    /// Compute ordered sum
    ///
    /// This function returns the correct result when |`self`| ≥ |`other`|.
    /// The result is not guaranteed to be correct when |`self`| < |`other`|.
    #[inline]
    const fn ordered_add(self, other: Self) -> Self {
        let pair = Self::from_ordered_sum(self.high, other.high);
        let low = self.low + other.low + pair.low;
        let high = pair.high;
        Self { high, low }
    }
}

impl Add for Pair {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        let pair = Self::from_sum(self.high, other.high);
        let low = self.low + other.low + pair.low;
        let high = pair.high;
        Self { high, low }
    }
}

impl Mul for Pair {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        let high = self.high * other.high;
        let low = crate::mul_add(self.low, other.high, self.high * other.low);
        let low = low + self.high.mul_add(other.high, -high);
        Self { high, low }
    }
}

impl Mul<f64> for Pair {
    type Output = Self;

    #[inline]
    fn mul(self, other: f64) -> Self {
        let high = self.high * other;
        let low = crate::mul_add(self.low, other, self.high.mul_add(other, -high));
        Self { high, low }
    }
}
