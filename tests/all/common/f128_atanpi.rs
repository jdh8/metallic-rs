//! Inputs and precision-113 MPFR oracles for the pi-scaled arc tangents.
#![allow(dead_code)]
use super::common128;

pub const MASK: u128 = (1 << 112) - 1;
pub const SIGN: u128 = 1 << 127;

pub fn power(e: i32) -> f128 {
    metallic::ldexpq(1.0, e)
}

pub fn neighbors(out: &mut Vec<f128>, x: f128, radius: u128) {
    let bits = x.to_bits() & !SIGN;
    for b in bits.saturating_sub(radius)..=bits + radius {
        out.extend([f128::from_bits(b), f128::from_bits(b | SIGN)]);
    }
}

/// Cover every exponent field, both signs, subnormals, infinities and NaNs.
pub fn domain(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | ((i as u128 % 0x8000) << 112) | b & MASK)
}

pub fn banded(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | ((16363 + (b >> 112) % 41) << 112) | b & MASK)
}

pub fn subnormal(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | b & ((1 << 115) - 1))
}

/// Table breakpoints, their reciprocals, sector boundaries, and tiny/huge seams.
pub fn seams(i: u64) -> f128 {
    let b = common128::mix128(i);
    let j = ((b >> 113) % 64 + 1) as f128;
    let near = match i % 6 {
        0 => j / 64.0,
        1 => 64.0 / j,
        2 => (j - 0.5) / 64.0,
        3 => 64.0 / (j - 0.5),
        _ => power([-7, -64, -192, 113, 114, 115][i as usize % 6]),
    };
    f128::from_bits(b & SIGN | near.to_bits().wrapping_add(b >> 120 & 15).wrapping_sub(7))
}

pub fn pairs(i: u64) -> [f128; 2] {
    let a = common128::mix128(2 * i);
    let b = common128::mix128(2 * i + 1);
    [f128::from_bits(a), f128::from_bits(b)]
}

pub fn banded_pairs(i: u64) -> [f128; 2] {
    [banded(2 * i), banded(2 * i + 1)]
}

pub fn seam_pairs(i: u64) -> [f128; 2] {
    let b = common128::mix128(i);
    let x = f128::from_bits(1.0_f128.to_bits() | b & MASK);
    let y = x * seams(i);
    [y, if b & (SIGN >> 1) == 0 { x } else { -x }]
}

/// Both signs on the subnormal-output grid, then swap towards the vertical axis.
pub fn tiny_pairs(i: u64) -> [f128; 2] {
    let y = subnormal(2 * i);
    let b = common128::mix128(2 * i + 1);
    let x = f128::from_bits(b & SIGN | ((16380 + (b >> 112) % 128) << 112) | b & MASK);
    if i & 1 == 0 { [y, x] } else { [x, y] }
}

pub fn edges() -> Vec<f128> {
    let mut out = vec![0.0, -0.0, f128::INFINITY, f128::NEG_INFINITY, f128::NAN];
    for x in [1.0, f128::MAX, f128::MIN_POSITIVE, 0.0] {
        neighbors(&mut out, x, 32);
    }
    for e in [
        -7, -8, -63, -64, -65, -191, -192, -193, 112, 113, 114, 115, 116,
    ] {
        neighbors(&mut out, power(e), 16);
    }
    for k in 0..113 {
        neighbors(&mut out, f128::from_bits(1 << k), 4);
    }
    for k in 1..=128 {
        let t = k as f128 / 128.0;
        neighbors(&mut out, t, 8);
        neighbors(&mut out, 1.0 / t, 8);
    }
    out
}

pub fn edge_pairs() -> Vec<[f128; 2]> {
    let special = [
        0.0_f128,
        -0.0,
        1.0,
        -1.0,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::NAN,
        f128::MAX,
        f128::MIN_POSITIVE,
        f128::from_bits(1),
        -f128::from_bits(1),
        f128::from_bits(MASK),
    ];
    let mut out: Vec<_> = special
        .into_iter()
        .flat_map(|y| special.map(|x| [y, x]))
        .collect();
    for v in edges() {
        for x in [1.0, -1.0] {
            out.extend([[v, x], [x, v]]);
        }
    }
    // Exact quarters and their neighbors at every normal exponent; equality
    // of two subnormals is included in the edge matrix above.
    for e in -16382..=16383 {
        let x = power(e);
        for y in [x.next_down(), x, x.next_up()] {
            out.extend([[y, x], [-y, x], [y, -x], [-y, -x]]);
        }
    }
    out
}

#[cfg(feature = "mpfr")]
pub fn oracle(x: f128) -> f128 {
    metallic::f128_mpfr::cr_unop(x, |y| y.atan_pi_round(rug::float::Round::Nearest))
}

#[cfg(feature = "mpfr")]
pub fn oracle2(y: f128, x: f128) -> f128 {
    metallic::f128_mpfr::cr_binop(y, x, |a, b| a.atan2_pi_round(b, rug::float::Round::Nearest))
}
