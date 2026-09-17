#![feature(f128)]
//! Freeze gamma edges, poles and roots, exact factorials, inverse midpoints,
//! and MPFR near-midpoint scans, independently of the implementation.
//!
//! CC=clang cargo +nightly run --release --features "f128 mpfr" \
//! --example gen_f128_gamma_cases -- <tgamma|lgamma> [scan-count]
//!
//! `audit-roots <path>` exports the nearest binary128 inputs of the negative
//! lgamma zeros. `extra-edges <path>` exports x/tgamma/lgamma triples for the
//! terminal subnormal boundaries, allowing an existing scan to be extended.

#[path = "support/f128_special.rs"]
mod support;
use rug::{Float, float::Round};
use support::*;

fn lgamma(x: &Float) -> Float {
    let mut y = x.clone();
    y.ln_abs_gamma_round(Round::Nearest);
    y
}
fn certify_inverse(x: &Float, target: &Float) {
    let error = (lgamma(x) - target).abs();
    let scale = target.clone().abs().max(&Float::with_val(PREC, 1));
    assert!(
        error < (scale >> 300_u32),
        "logGamma inverse did not converge: x={}, target={}, residual={}",
        hex(x.to_f128()),
        target.to_f64(),
        error.to_f64()
    );
}
/// Invert logGamma on its increasing positive branch. The target is kept in
/// log space even for gamma midpoints near the binary128 overflow threshold.
fn positive_inverse(target: Float) -> Float {
    let mut x = if target > 4 {
        Float::with_val(PREC, &target / (target.clone().ln() - 1)) + 2
    } else {
        Float::with_val(PREC, 2) + &target
    };
    for _ in 0..18 {
        let step = (lgamma(&x) - &target) / x.clone().digamma();
        x -= step;
    }
    certify_inverse(&x, &target);
    x
}
/// The root near one has a negative derivative. All requested targets are
/// tiny, so Newton remains strictly inside the interval (1/2, 3/2).
fn near_one_inverse(target: Float) -> Float {
    let mut x = Float::with_val(PREC, 1);
    for _ in 0..10 {
        x -= (lgamma(&x) - &target) / x.clone().digamma();
    }
    certify_inverse(&x, &target);
    x
}
/// Bracket both log|Gamma| zeros in (-n-1,-n), n >= 2. A bisection gets
/// below every pole distance that has distinct binary128 neighbours; Newton
/// then supplies the 512-bit root used by the midpoint inverse family.
fn negative_root(n: u32, left: bool) -> Float {
    negative_inverse(n, left, &Float::with_val(PREC, 0))
}
fn negative_inverse(n: u32, left: bool, target: &Float) -> Float {
    let pole = -Float::with_val(PREC, if left { n + 1 } else { n });
    let middle = -Float::with_val(PREC, n) - 0.5;
    let (mut lo, mut hi) = if left { (pole, middle) } else { (middle, pole) };
    for _ in 0..192 {
        let x = Float::with_val(PREC, &lo + &hi) / 2;
        if (lgamma(&x) > *target) == left {
            lo = x;
        } else {
            hi = x;
        }
    }
    let mut x = Float::with_val(PREC, &lo + &hi) / 2;
    for _ in 0..6 {
        x -= (lgamma(&x) - target) / x.clone().digamma();
    }
    certify_inverse(&x, target);
    x
}
/// Invert the actual IEEE underflow and normal/subnormal boundaries on both
/// sides of every relevant negative pole. At n=1774 the last nonzero values
/// are only a few binary128 steps from the pole; half-integer sampling cannot
/// find this family. The same roots target the first odd subnormal midpoints.
fn extra_edges() -> Vec<f128> {
    let mut out = Vec::new();
    neighbors(&mut out, 31.0 / 32.0, 16);
    let targets: Vec<_> = [1, 3, 5, 127, (1_u128 << 113) - 1, (1_u128 << 113) + 1]
        .map(|m| ((Float::with_val(PREC, m)) >> 16495_u32).ln())
        .into();
    for n in 1740..=1776 {
        let middle = lgamma(&(-Float::with_val(PREC, n) - 0.5));
        for target in &targets {
            if middle >= *target {
                continue;
            }
            for left in [false, true] {
                // Once the pole residue's inverse lies far inside its input
                // half-ulp, the inverse rounds to the pole itself. Avoid
                // requesting hundreds of irrelevant cancellation bits there.
                let pole = if left { n + 1 } else { n };
                let distance = (-lgamma(&Float::with_val(PREC, pole + 1)) - target).exp();
                if distance < power(-106) {
                    neighbors(&mut out, -(pole as f128), 3);
                    continue;
                }
                neighbors(&mut out, negative_inverse(n, left, target).to_f128(), 3);
            }
        }
    }
    out
}
fn roots() -> Vec<Float> {
    let mut roots = Vec::new();
    let mut closest = Float::with_val(PREC, 1);
    let mut closest_x = 0.0_f128;
    for n in 2..=40 {
        for left in [false, true] {
            let x = negative_root(n, left);
            let rounded = x.to_f128();
            if rounded != rounded.trunc() {
                for input in [rounded.next_down(), rounded, rounded.next_up()] {
                    let value = lgamma(&Float::with_val(PREC, input)).abs();
                    if value < closest {
                        closest = value;
                        closest_x = input;
                    }
                }
                roots.push(x);
            }
        }
    }
    eprintln!(
        "negative lgamma roots: {} representable neighborhoods; min |lgamma(x)|={} at {}",
        roots.len(),
        closest.to_f64(),
        hex(closest_x)
    );
    roots
}
fn edges(roots: &[Float]) -> Vec<f128> {
    let mut out = common_edges();
    // Every positive integer through overflow (including the factorial's
    // exact/midpoint subset) and every negative pole in the nonzero band.
    for n in 1..=1800 {
        neighbors(&mut out, n as f128, 4);
        neighbors(&mut out, n as f128 + 0.5, 2);
    }
    for j in 0..=16 {
        neighbors(&mut out, 1.0 + j as f128 / 16.0, 16);
    }
    neighbors(&mut out, 64.0, 64);
    neighbors(&mut out, 31.0 / 32.0, 16);
    neighbors(&mut out, metallic::ldexpq(1.0, -256), 32);
    for x in roots {
        neighbors(&mut out, x.to_f128(), 32);
    }
    // The positive overflow boundary of gamma and lgamma. MPFR retains the
    // half-ulp beyond MAX rather than rounding the boundary prematurely.
    let overflow = power(16384) - power(16270);
    neighbors(
        &mut out,
        positive_inverse(overflow.clone().ln()).to_f128(),
        32,
    );
    neighbors(&mut out, positive_inverse(overflow).to_f128(), 32);
    // Tiny gamma changes from overflow to a reciprocal; both signs retain
    // the Euler-constant correction on either side of this boundary.
    let reciprocal_overflow = (power(16384) - power(16270)).recip();
    neighbors(&mut out, reciprocal_overflow.to_f128(), 32);
    out
}
fn inverse_midpoints(kind: usize, roots: &[Float]) -> Vec<f128> {
    let mut out = Vec::new();
    // The increasing branch covers all finite gamma values >=1, all large
    // positive lgamma values, and arbitrarily small lgamma values near 2.
    let low: i32 = if kind == 2 { 0 } else { -120 };
    for e in low..=16383 {
        if e > 128 && e % 64 != 0 && e != 16383 {
            continue;
        }
        let samples = if e > 128 { 2 } else { 16 };
        for i in 0..samples {
            let m = (1 << 113) | ((bits(((e - low) * 32 + i) as u64) & MASK) << 1) | 1;
            let z = Float::with_val(PREC, m) << (e - 113);
            let target = if kind == 2 { z.ln() } else { z };
            neighbors(&mut out, positive_inverse(target.clone()).to_f128(), 1);
            if target.clone().abs() < 0.0625 {
                neighbors(&mut out, near_one_inverse(target.clone()).to_f128(), 1);
                neighbors(&mut out, positive_inverse(-target.clone()).to_f128(), 1);
                neighbors(&mut out, near_one_inverse(-target).to_f128(), 1);
            }
        }
    }
    // Midpoints adjacent to gamma=1 (both positive roots); for lgamma the
    // same family is already generated in the small-output binades above.
    if kind == 2 {
        for i in 0..1024 {
            for e in [-114_i32, -113] {
                let d = Float::with_val(PREC, 2 * i + 1) << e;
                for z in [Float::with_val(PREC, 1) + &d, Float::with_val(PREC, 1) - &d] {
                    let target = z.ln();
                    neighbors(&mut out, positive_inverse(target.clone()).to_f128(), 1);
                    neighbors(&mut out, near_one_inverse(target).to_f128(), 1);
                }
            }
        }
    }
    // Negative lgamma zeros are cancellation points unrelated to the exact
    // roots +1 and +2. Invert signed output midpoints around every root with
    // distinct binary128 neighbours. For gamma this targets ±1 midpoints.
    for (j, root) in roots.iter().enumerate() {
        for e in (-120_i32..=-4).step_by(4) {
            for i in 0..4 {
                let m = (1 << 113)
                    | ((bits((j * 512 + (e + 120) as usize * 4 + i) as u64) & MASK) << 1)
                    | 1;
                let z = Float::with_val(PREC, m) << (e - 113);
                for target in [z.clone(), -z] {
                    let target = if kind == 2 {
                        (Float::with_val(PREC, 1) + target).ln()
                    } else {
                        target
                    };
                    let mut x = root.clone();
                    for _ in 0..6 {
                        x -= (lgamma(&x) - &target) / x.clone().digamma();
                    }
                    neighbors(&mut out, x.to_f128(), 1);
                }
            }
        }
    }
    out
}
/// At a negative pole -n, |Gamma(-n+delta)| ~ 1/(n! |delta|).
/// Inverting each branch's log magnitude chooses subnormal output midpoints
/// and normal midpoints down to underflow without underflowing MPFR itself.
fn reflection_midpoints() -> Vec<f128> {
    let mut out = Vec::new();
    for n in (2..=1760).step_by(3) {
        let start = -Float::with_val(PREC, n) - 0.25;
        let value = lgamma(&start).exp();
        let center = value.to_f128();
        if center == 0.0 || !center.is_finite() {
            continue;
        }
        let z = (Float::with_val(PREC, center) + Float::with_val(PREC, center.next_up())) / 2_u32;
        let target = z.ln();
        let mut x = start;
        for _ in 0..8 {
            x -= (lgamma(&x) - &target) / x.clone().digamma();
        }
        neighbors(&mut out, x.to_f128(), 2);
    }
    // Entire terminal binades and their subnormal output grid.
    for n in 1700..=1770 {
        for i in 0..128 {
            neighbors(&mut out, -(n as f128) - (i as f128 + 0.5) / 128.0, 2);
        }
    }
    out
}
fn sample(i: u64) -> f128 {
    let b = bits(i);
    match i % 4 {
        0 => f128::from_bits(b & SIGN | (16383 - 260 + (i / 4 % 272) as u128) << 112 | b & MASK),
        1 => {
            let x = (b & MASK) as f128 * metallic::ldexpq(1800.0, -112);
            if b & SIGN == 0 { x } else { -x }
        }
        2 => {
            let n = (i / 4 % 1800 + 1) as f128;
            let delta = (b & ((1_u128 << (1 + i / 7200 % 112)) - 1)).max(1);
            -f128::from_bits(if b & SIGN == 0 {
                n.to_bits() + delta
            } else {
                n.to_bits() - delta
            })
        }
        _ => {
            let center = if i % 8 == 3 { 1.0_f128 } else { 2.0 };
            let delta = (b & ((1_u128 << (1 + i / 8 % 112)) - 1)).max(1);
            f128::from_bits(if b & SIGN == 0 {
                center.to_bits() + delta
            } else {
                center.to_bits() - delta
            })
        }
    }
}
fn main() {
    let name = std::env::args().nth(1).expect("tgamma or lgamma");
    if name == "audit-roots" {
        let path = std::env::args().nth(2).expect("audit-roots output path");
        let text = roots()
            .iter()
            .map(|x| format!("{:032x}\n", x.to_f128().to_bits()))
            .collect::<String>();
        std::fs::write(path, text).unwrap();
        return;
    }
    if name == "extra-edges" {
        use std::fmt::Write as _;
        let path = std::env::args().nth(2).expect("extra-edges output path");
        let mut text = String::new();
        let mut seen = std::collections::HashSet::new();
        for x in extra_edges() {
            if seen.insert(x.to_bits()) {
                let gamma = metallic::f128_mpfr::cr_unop(x, |y| operation(2, y));
                let log = metallic::f128_mpfr::cr_unop(x, |y| operation(3, y));
                writeln!(text, "{} {} {}", hex(x), hex(gamma), hex(log)).unwrap();
            }
        }
        std::fs::write(path, text).unwrap();
        return;
    }
    let kind = 2 + ["tgamma", "lgamma"]
        .iter()
        .position(|&s| s == name)
        .expect("unknown function");
    let count = std::env::args()
        .nth(2)
        .map_or(20_000_000, |s| s.parse().unwrap());
    let command = format!(
        "CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_gamma_cases -- {name} {count}"
    );
    let roots = roots();
    let edges = edges(&roots);
    eprintln!("gamma edges: {} inputs before deduplication", edges.len());
    let inverse = inverse_midpoints(kind, &roots);
    eprintln!(
        "gamma inverse midpoints: {} inputs before deduplication",
        inverse.len()
    );
    let reflection = reflection_midpoints();
    let sections = vec![
        (
            "exact factorials, poles, exponent fields, overflow edges, table seams and negative lgamma zeros",
            edges,
        ),
        (
            "inverse 114-bit output midpoints, including positive and negative lgamma roots",
            inverse,
        ),
        (
            "negative reflection inverse midpoints and the terminal subnormal band",
            reflection,
        ),
        (
            "full-representation regression sample",
            (0..65536).map(domain).collect(),
        ),
        (
            "MPFR near-midpoint scan in four active bands",
            scan(kind, count, sample),
        ),
        (
            "inverse subnormal midpoint boundaries at the terminal negative poles and direct-Taylor seam",
            extra_edges(),
        ),
    ];
    write_corpus(kind, &command, sections);
}
