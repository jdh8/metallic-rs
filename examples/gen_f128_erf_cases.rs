#![feature(f128)]
//! Freeze error-function edges, inverse midpoints, tiny-slope convergents, and
//! MPFR near-midpoint scans. No implementation output enters the corpus.
//!
//! CC=clang cargo +nightly run --release --features "f128 mpfr" \
//! --example gen_f128_erf_cases -- <erf|erfc> [scan-count]
//!
//! `audit-inverses` checks every analytic inverse family without scanning.

#[path = "support/f128_special.rs"]
mod support;
use rug::{Float, Integer, float::Constant};
use support::*;

fn certify_inverse(x: &Float, target: &Float, complement: bool) {
    let value = if complement {
        x.clone().erfc()
    } else {
        x.clone().erf()
    };
    let error = (value - target).abs();
    assert!(
        error == 0 || error < (target.clone().abs() >> 300_u32),
        "error-function inverse did not converge: complement={complement}, x={}, target={}, residual={}",
        x.to_f64(),
        target.to_f64(),
        error.to_f64()
    );
}
fn inverse_erf(z: Float) -> Float {
    if z > 0.5 {
        return inverse_erfc(Float::with_val(PREC, 1) - z);
    }
    let slope_inverse = Float::with_val(PREC, Constant::Pi).sqrt() / 2;
    let mut x = Float::with_val(PREC, &z * &slope_inverse);
    for _ in 0..10 {
        let residual = Float::with_val(PREC, x.clone().erf() - &z);
        let derivative_inverse = x.clone().square().exp() * &slope_inverse;
        x -= residual * derivative_inverse;
    }
    certify_inverse(&x, &z, false);
    x
}
fn inverse_erfc(z: Float) -> Float {
    if z > 1 {
        return -inverse_erfc(Float::with_val(PREC, 2) - z);
    }
    if z > 0.5 {
        return inverse_erf(Float::with_val(PREC, 1) - z);
    }
    let pi = Float::with_val(PREC, Constant::Pi);
    let t = -z.clone().ln();
    let mut square = t.clone() - t.clone().ln() / 2 - pi.clone().ln() / 2;
    if square < 0.125 {
        square = Float::with_val(PREC, 0.125);
    }
    let mut x = square.sqrt();
    for _ in 0..16 {
        let residual = Float::with_val(PREC, x.clone().erfc() - &z);
        x += residual * (x.clone().square().exp() * pi.clone().sqrt() / 2);
    }
    certify_inverse(&x, &z, true);
    x
}
fn edges() -> Vec<f128> {
    let mut out = common_edges();
    for e in -1..=6 {
        for j in 0..=16 {
            neighbors(&mut out, metallic::ldexpq((16 + j) as f128, e - 4), 8);
        }
    }
    for x in [8.0, 9.0, 10.0, 16.0, 106.0, 107.0, 108.0] {
        neighbors(&mut out, x, 32);
    }
    // First rounding to erf=1, negative erfc=2, zero erfc, and minimum normal.
    for e in [-114, -113, -16495, -16382] {
        neighbors(&mut out, inverse_erfc(power(e)).to_f128(), 32);
    }
    out
}
fn inverse_midpoints(kind: usize) -> Vec<f128> {
    let mut out = Vec::new();
    let low = if kind == 0 { -160 } else { -16494 };
    for e in low..=0 {
        let samples = if e < -256 { 1 } else { 32 };
        for i in 0..samples {
            let m = (1 << 113) | ((bits(((e - low) * 32 + i) as u64) & MASK) << 1) | 1;
            let z = Float::with_val(PREC, m) << (e - 113);
            if z >= if kind == 0 { 1 } else { 2 } {
                continue;
            }
            let x = if kind == 0 {
                inverse_erf(z)
            } else {
                inverse_erfc(z)
            };
            neighbors(&mut out, x.to_f128(), 1);
        }
    }
    for i in 0..2048 {
        let odd = 2 * i + 1;
        // Both subnormal output grids, including the underflow midpoint.
        let z = Float::with_val(PREC, odd) >> 16495;
        let x = if kind == 0 {
            inverse_erf(z)
        } else {
            inverse_erfc(z)
        };
        neighbors(&mut out, x.to_f128(), 1);
        // Saturation at one, and the unequal grids immediately around one.
        for e in [-114, -113] {
            let delta = Float::with_val(PREC, odd) << e;
            let z = Float::with_val(PREC, 1) - &delta;
            let x = if kind == 0 {
                inverse_erf(z)
            } else {
                inverse_erfc(z)
            };
            neighbors(&mut out, x.to_f128(), 1);
            if kind == 1 {
                neighbors(
                    &mut out,
                    inverse_erfc(Float::with_val(PREC, 1) + delta).to_f128(),
                    1,
                );
            }
        }
    }
    out
}
/// In the tiny erf band, y/ulp is m*(2/sqrt(pi)) or m/sqrt(pi).
/// Odd numerators of convergents to twice either slope approach midpoints;
/// odd multiples lift short convergents into the 113-bit significand lattice.
fn linear_family() -> Vec<f128> {
    let mut out = Vec::new();
    for numerator in [2, 4] {
        let slope = Float::with_val(640, numerator) / Float::with_val(640, Constant::Pi).sqrt();
        let mut a = (slope << 600_u32).to_integer().unwrap();
        let mut b = Integer::from(1) << 600_u32;
        let (mut p0, mut q0) = (Integer::from(0), Integer::from(1));
        let (mut p1, mut q1) = (Integer::from(1), Integer::from(0));
        loop {
            let (c, rem) = a.div_rem(b.clone());
            let p = Integer::from(&c * &p1) + &p0;
            let q = Integer::from(&c * &q1) + &q0;
            if q.significant_bits() > 113 {
                break;
            }
            if p.is_odd() && q > 0 {
                let q = q.to_u128().unwrap();
                let multiplier = (1_u128 << 112).div_ceil(q) | 1;
                for m in [q, q * multiplier] {
                    if m >= 1 << 113 {
                        continue;
                    }
                    if numerator == 4 {
                        neighbors(&mut out, f128::from_bits(m), 2);
                    }
                    for e in [-16382_i32, -1024, -256, -192, -128, -114, -96, -80, -64] {
                        neighbors(
                            &mut out,
                            (Float::with_val(113, m) << (e - 112)).to_f128(),
                            2,
                        );
                    }
                }
            }
            (p0, q0, p1, q1) = (p1, q1, p, q);
            if rem == 0 {
                break;
            }
            a = b;
            b = rem;
        }
    }
    out
}
fn sample(i: u64) -> f128 {
    let b = bits(i);
    match i % 4 {
        0 => f128::from_bits(b & SIGN | (16383 - 120 + (i / 4 % 128) as u128) << 112 | b & MASK),
        1 => f128::from_bits(b & SIGN | b & ((1 << 114) - 1)),
        2 => (b & MASK) as f128 * metallic::ldexpq(108.0, -112),
        _ => f128::from_bits(b & SIGN | (16383 - 4 + (i / 4 % 8) as u128) << 112 | b & MASK),
    }
}
fn main() {
    let name = std::env::args().nth(1).expect("erf or erfc");
    if name == "audit-inverses" {
        edges();
        for kind in 0..2 {
            let cases = inverse_midpoints(kind);
            eprintln!(
                "kind={kind}: certified all inverse families ({} inputs before deduplication)",
                cases.len()
            );
        }
        return;
    }
    let kind = ["erf", "erfc"]
        .iter()
        .position(|&s| s == name)
        .expect("unknown function");
    let count = std::env::args()
        .nth(2)
        .map_or(20_000_000, |s| s.parse().unwrap());
    let command = format!(
        "CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_erf_cases -- {name} {count}"
    );
    let sections = vec![
        (
            "exact cases, all exponent fields, subnormal and saturation thresholds, table seams",
            edges(),
        ),
        (
            "inverse 114-bit output midpoints and the subnormal result grid",
            inverse_midpoints(kind),
        ),
        (
            "continued fractions of the tiny irrational slope on both IEEE grids",
            linear_family(),
        ),
        (
            "full-representation regression sample",
            (0..65536).map(domain).collect(),
        ),
        (
            "MPFR near-midpoint scan in four active bands",
            scan(kind, count, sample),
        ),
    ];
    write_corpus(kind, &command, sections);
}
