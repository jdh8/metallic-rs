#![feature(f128)]
//! Freeze the π-scaled binary128 corpora with precision-113 MPFR answers.
//!
//! Three layers: exact values and every reduction seam (including subnormal
//! results); inverse images of 114-bit output midpoints plus continued
//! fractions of π and π/2 for the tiny linear band; and a near-midpoint scan
//! over the active band with a full-representation regression sample.
//!
//! `CC=clang cargo +nightly run --release --features "f128 mpfr"
//! --example gen_f128_trigpi_cases -- 20000000` (scan count per function).

#[path = "../tests/all/common/f128_trigpi.rs"]
mod cases;
#[path = "../tests/all/common.rs"]
mod common;
#[path = "../tests/all/common/f128.rs"]
mod common128;

use cases::{MASK as MANTISSA, SIGN};
use rug::{
    Float, Integer,
    float::{Constant, Round},
};
use std::fmt::Write as _;
const PREC: u32 = 300;
const THRESHOLD: f64 = 1.0 / 16384.0;
const THREADS: u64 = 4;

/// CORE-MATH's hexadecimal spelling of a binary128 value.
fn hex(x: f128) -> String {
    let bits = x.to_bits();
    let sign = if bits & SIGN == 0 { "" } else { "-" };
    let exponent = (bits >> 112 & 0x7fff) as i32;
    let mantissa = bits & MANTISSA;

    match (exponent, mantissa) {
        (0x7fff, 0) => format!("{sign}inf"),
        (0x7fff, _) => format!("{sign}nan"),
        (0, 0) => format!("{sign}0x0p+0"),
        (0, _) => format!("{sign}0x0.{mantissa:028x}p-16382"),
        _ => format!("{sign}0x1.{mantissa:028x}p{:+}", exponent - 16383),
    }
}
/// Normalized distance from `y` to the nearest binary128 midpoint.
///
/// Round-to-nearest is symmetric, so the magnitude decides: stepping the bit
/// pattern of `|c|` up and down is then the neighbouring grid.
fn midpoint_frac(y: &Float) -> f64 {
    let y = &Float::with_val(PREC, y.abs_ref());
    let c = y.to_f128_round(Round::Nearest);
    if !c.is_finite() || c == 0.0 {
        return 1.0;
    }
    let step = |bits: u128| f128::from_bits(bits);
    let up = step(c.to_bits() + 1);
    let down = step(c.to_bits() - 1);
    let mid_hi = (Float::with_val(PREC, c) + Float::with_val(PREC, up)) / 2u32;
    let mid_lo = (Float::with_val(PREC, c) + Float::with_val(PREC, down)) / 2u32;
    let d_hi = Float::with_val(PREC, y - &mid_hi).abs();
    let d_lo = Float::with_val(PREC, y - &mid_lo).abs();
    let c = Float::with_val(PREC, c);

    if d_hi < d_lo {
        (d_hi / (mid_hi - c)).to_f64()
    } else {
        (d_lo / (c - mid_lo)).to_f64()
    }
}

/// Hard outputs first: odd 114-bit significands, then the inverse rounded
/// to binary128 with its two neighbors. Cosine additionally targets the
/// midpoints immediately below 1, where its quadratic term first rounds.
fn inverse(kind: usize) -> Vec<f128> {
    let mut out = Vec::new();
    let lo = if kind == 1 { -120 } else { -16494 };
    let hi = if kind == 2 { 112 } else { -1 };
    for e in lo..=hi {
        // Deep in the linear band, one midpoint per binade supplements
        // the much harder continued-fraction family.
        let samples = if (-16379..-160).contains(&e) { 1 } else { 8 };
        for k in 0..samples {
            let h = common128::mix128(((e + 16494) * 8 + k) as u64);
            let m = (1 << 113) | ((h & MANTISSA) << 1) | 1;
            let z = Float::with_val(PREC, m) << (e - 113);
            inverse_one(&mut out, kind, z);
        }
    }
    if kind != 2 {
        for k in 0..2048u32 {
            let z = Float::with_val(PREC, 1) - (Float::with_val(PREC, 2 * k + 1) >> 114);
            inverse_one(&mut out, kind, z);
        }
    }
    out
}

fn inverse_one(out: &mut Vec<f128>, kind: usize, mut z: Float) {
    match kind {
        0 => z.asin_pi_mut(),
        1 => z.acos_pi_mut(),
        _ => z.atan_pi_mut(),
    }
    let x = z.to_f128();
    cases::neighbors(out, x, 1);
    // Reflections exercise both sides of the other zeros/poles too.
    if x >= cases::power(-114) {
        cases::neighbors(out, 1.0 - x, 1);
        cases::neighbors(out, 1.0 + x, 1);
    }
}

/// For tiny inputs the odd functions round π·x. Convergents p/q of π or
/// π/2 with odd p put that product near a binary128 midpoint. Odd multiples
/// preserve the midpoint parity while filling the 113-bit significand.
fn linear_family() -> Vec<f128> {
    let mut out = Vec::new();
    for divisor in [1u32, 2] {
        let value = (Float::with_val(400, Constant::Pi) / divisor) << 384u32;
        let mut a = value.to_integer().unwrap();
        let mut b = Integer::from(1) << 384u32;
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
                let multiplier = (((1u128 << 112) + q - 1) / q) | 1;
                let m = q * multiplier;
                if m < 1 << 113 {
                    for e in [-16382i32, -1024, -256, -160, -128, -114, -96, -80, -67, -66] {
                        let x = (Float::with_val(113, m) << (e - 112)).to_f128();
                        cases::neighbors(&mut out, x, 2);
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

fn scan(kind: usize, count: u64) -> Vec<f128> {
    let mut kept = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..THREADS)
            .map(|thread| {
                scope.spawn(move || {
                    let mut out = Vec::new();
                    for i in (thread..count).step_by(THREADS as usize) {
                        let x = match i % 3 {
                            0 => cases::band(i, -67, 20),
                            1 => cases::near_grid(i),
                            _ => cases::band(i, -9, 103),
                        };
                        let mut y = Float::with_val(PREC, x);
                        cases::operation(kind, &mut y);
                        if midpoint_frac(&y) < THRESHOLD {
                            out.push(x);
                        }
                        if thread == 0 && i % 1_000_000 == 0 {
                            eprintln!("kind={kind} scan: {i}/{count}");
                        }
                    }
                    out
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect::<Vec<_>>()
    });
    kept.sort_by_key(|x| x.to_bits());
    kept
}

fn main() {
    let count = std::env::args()
        .nth(1)
        .map_or(20_000_000, |s| s.parse().expect("scan count"));
    let edges = cases::edges();
    let linear = linear_family();
    let wide: Vec<_> = (0..8192).map(cases::wide).collect();
    for (kind, name) in ["sinpiq", "cospiq", "tanpiq"].into_iter().enumerate() {
        let inverse = inverse(kind);
        let scan = scan(kind, count);
        let mut text = format!(
            "# {name}(x): MPFR precision-113 answers, with ternary-aware IEEE subnormalization.\n\
             # Generated by `CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_trigpi_cases -- {count}`.\n"
        );
        let mut seen = std::collections::HashSet::new();
        let mut mismatches = 0;
        for (title, inputs) in [
            (
                "exact cases, subnormal edges, table and integer/half-integer seams",
                &edges,
            ),
            (
                "continued fractions of pi and pi/2 in the tiny linear band",
                &linear,
            ),
            (
                "inverse images of 114-bit rounding midpoints with neighbors",
                &inverse,
            ),
            ("full-representation regression sample", &wide),
            ("near-midpoints from the MPFR active-band scan", &scan),
        ] {
            writeln!(text, "#\n# {title}\n#").unwrap();
            for &x in inputs {
                if !seen.insert(x.to_bits()) {
                    continue;
                }
                let want = cases::oracle(kind, x);
                let got = cases::FUNCTIONS[kind](x);
                if got.to_bits() != want.to_bits() && !(got.is_nan() && want.is_nan()) {
                    if mismatches < 20 {
                        eprintln!("{name}({x:?}) = {got:?}, want {want:?}");
                    }
                    mismatches += 1;
                }
                writeln!(text, "{} {}", hex(x), hex(want)).unwrap();
            }
        }
        let path = format!("tests/cases/{name}.wc");
        std::fs::write(&path, text).unwrap();
        eprintln!(
            "wrote {path}: {} cases, {} near-midpoints, {mismatches} mismatches",
            seen.len(),
            scan.len()
        );
        assert_eq!(mismatches, 0);
    }
}
