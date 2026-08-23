//! Samplers shared by the binary128 exponential tests.
#![allow(dead_code)]

use crate::common128;

const SAMPLE_COUNT: u64 = 200_000;

/// Uniform dense sweep of `[lo, hi]`, which each test widens past its own
/// overflow and underflow thresholds.
pub fn dense(lo: f128, hi: f128) -> impl Iterator<Item = f128> {
    let step = (hi - lo) / SAMPLE_COUNT as f128;
    (0..=SAMPLE_COUNT).map(move |i| lo + i as f128 * step)
}

/// Random significands across every exponent the argument reduction sees,
/// `[2^-121, 2^15)`, plus raw bit patterns for the saturating ends.
pub fn significands() -> impl Iterator<Item = f128> {
    (0..SAMPLE_COUNT)
        .map(|i| {
            let bits = common128::mix128(i);
            let exponent = 16_383 - 121 + (bits >> 120) % 137;
            f128::from_bits((bits & 1 << 127) | exponent << 112 | (bits & (1 << 112) - 1))
        })
        .chain((0..SAMPLE_COUNT).map(|i| f128::from_bits(common128::mix128(i ^ 0x5555_5555))))
}

/// [`significands`] restricted to `[-bound, bound]`, for the MPFR sweeps.
pub fn sampler(bound: f128) -> impl Fn(u64) -> f128 {
    move |i| {
        let unit = (common128::mix128(i) >> 15) as f128 / (1_u128 << 113) as f128;
        (2.0 * unit - 1.0) * bound
    }
}
