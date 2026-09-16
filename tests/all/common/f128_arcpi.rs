//! Shared inputs and MPFR oracles for the pi-scaled inverse sine/cosine.
#![allow(dead_code)]

use super::common128;

pub const MASK: u128 = (1 << 112) - 1;
pub const SIGN: u128 = 1 << 127;
pub const FUNCTIONS: [fn(f128) -> f128; 2] = [metallic::asinpiq, metallic::acospiq];

pub fn power(e: i32) -> f128 {
    metallic::ldexpq(1.0, e)
}

pub fn neighbors(out: &mut Vec<f128>, x: f128, radius: u128) {
    let bits = x.to_bits() & !SIGN;
    for b in bits.saturating_sub(radius)..=bits + radius {
        out.push(f128::from_bits(b));
        out.push(f128::from_bits(b | SIGN));
    }
}

/// Every exponent field in the domain, including subnormals and both signs.
pub fn domain(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | ((i as u128 % 0x3fff) << 112) | b & MASK)
}

pub fn dense(i: u64) -> f128 {
    let b = common128::mix128(i);
    let x = (b & MASK) as f128 * power(-112);
    if b & SIGN != 0 { -x } else { x }
}

pub fn near_one(i: u64) -> f128 {
    let b = common128::mix128(i);
    let offset = 1 + ((b & MASK) >> ((b >> 113) % 112));
    f128::from_bits(b & SIGN | (1.0_f128.to_bits() - offset))
}

pub fn subnormal(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | b & ((1 << 115) - 1))
}

/// Polynomial bands, cubic significance, tiny acos rounding, and root sectors.
pub fn seams(i: u64) -> f128 {
    let b = common128::mix128(i);
    let j = (b >> 113) as u32 % 91 + 1;
    let t = j as f128 / 128.0;
    let x = match i % 4 {
        0 => t,
        1 => metallic::sqrtq(1.0 - t * t),
        _ => power([-3, -4, -57, -63, -64, -114, -115][j as usize % 7]),
    };
    f128::from_bits(b & SIGN | x.to_bits().wrapping_add(b >> 120 & 15).wrapping_sub(7))
}

pub fn small(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | ((16383 - 130 + (b >> 113) % 128) << 112) | b & MASK)
}

pub fn edges() -> Vec<f128> {
    let mut out = vec![0.0, -0.0, f128::NAN, f128::INFINITY, f128::NEG_INFINITY];
    for x in [
        1.0,
        0.5,
        power(-3),
        power(-4),
        power(-57),
        power(-63),
        power(-64),
        power(-114),
        power(-115),
        f128::MIN_POSITIVE,
        0.0,
    ] {
        neighbors(&mut out, x, 32);
    }
    for k in 0..113 {
        neighbors(&mut out, 1.0 - power(-k - 1), 4);
        neighbors(&mut out, f128::from_bits(1 << k), 4);
    }
    for j in 1..=91 {
        let t = j as f128 / 128.0;
        neighbors(&mut out, t, 8);
        neighbors(&mut out, metallic::sqrtq(1.0 - t * t), 8);
    }
    for j in 1..=64 {
        let t = j as f128 / 64.0;
        let c = metallic::rsqrtq(1.0 + t * t);
        neighbors(&mut out, c, 4);
        neighbors(&mut out, t * c, 4);
    }
    out
}

#[cfg(feature = "mpfr")]
pub fn operation(kind: usize, y: &mut rug::Float) -> core::cmp::Ordering {
    use rug::float::Round::Nearest;
    if kind == 0 {
        y.asin_pi_round(Nearest)
    } else {
        y.acos_pi_round(Nearest)
    }
}

#[cfg(feature = "mpfr")]
pub fn oracle(kind: usize, x: f128) -> f128 {
    metallic::f128_mpfr::cr_unop(x, |y| operation(kind, y))
}
