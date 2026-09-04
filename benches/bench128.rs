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

/// [`bench::Exponents`] for `f128`: a random sign and mantissa with the exponent
/// drawn uniformly from the range.  Unlike the `f64` version this assembles the
/// bit pattern directly — an `f128` multiply is soft-float and would cost as much
/// as the function under test.
impl crate::bench::Draw<f128> for crate::bench::Exponents {
    fn draw(self) -> f128 {
        let e = rand::random_range(self.0);
        let bits = rand::random::<u128>();
        f128::from_bits((bits & SIGN) | (((e + 16383) as u128) << 112) | (bits & FRACTION))
    }
}

/// Positive-only counterpart used for real powers with noninteger exponents.
impl crate::bench::Draw<f128> for crate::bench::PositiveExponents {
    fn draw(self) -> f128 {
        let e = rand::random_range(self.0);
        let bits = rand::random::<u128>();
        f128::from_bits((((e + 16383) as u128) << 112) | (bits & FRACTION))
    }
}
