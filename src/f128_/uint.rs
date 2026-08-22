// This module is compiled only by the nightly-only `f128` feature.
#![allow(clippy::incompatible_msrv)]

/// Full unsigned 128×128-bit product as `(high, low)`.
#[must_use]
#[inline]
pub fn wmul(x: u128, y: u128) -> (u128, u128) {
    let (low, high) = x.carrying_mul(y, 0);
    (high, low)
}

/// High half of an unsigned 128×128-bit product.
#[must_use]
#[inline]
pub fn mhi(x: u128, y: u128) -> u128 {
    x.carrying_mul(y, 0).1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_product() {
        assert_eq!(wmul(u128::MAX, u128::MAX), (u128::MAX - 1, 1));
        assert_eq!(mhi(u128::MAX, u128::MAX), u128::MAX - 1);
    }
}
