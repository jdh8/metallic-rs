#![allow(dead_code)]

const SIGN: u128 = 1 << 127;
const FRACTION: u128 = (1 << 112) - 1;
const FINITE_EXPONENTS: u128 = 0x7FFE;

fn normal_bits() -> u128 {
    let bits = rand::random::<u128>();
    ((((bits >> 112) & 0x7FFF) % FINITE_EXPONENTS) + 1) << 112 | (bits & FRACTION)
}

/// Uniform exponent field and fraction over positive normal binary128 values.
pub fn positive_normal() -> f128 {
    f128::from_bits(normal_bits())
}

/// Uniform sign, exponent field, and fraction over normal binary128 values.
pub fn normal() -> f128 {
    let sign = rand::random::<u128>() & SIGN;
    f128::from_bits(sign | normal_bits())
}
