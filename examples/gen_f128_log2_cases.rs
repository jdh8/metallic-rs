#![feature(f128)]
//! Generate `tests/cases/log2q.wc`: the hard-to-round corpus for
//! [`metallic::log2q`], each input with its correctly rounded answer.
//!
//! CORE-MATH has no binary128 `log2` yet, so MPFR is the oracle and the answers
//! travel with the inputs; the strict gate then replays under plain
//! `--features f128` with no oracle at all.  Three layers:
//!
//! 1. **Edges.** Specials, the neighbours of 1, the fast leg's floor
//!    `|log2 x| = 2^-16`, every power of two (the exact cases, subnormals
//!    included) and the neighbours of every 32nd one, where the frame's
//!    integer part swamps a 2^-112 fraction; and significands within a few
//!    ulps of the reduction's table reciprocals `2^(j/2^18)`, where the
//!    reduced `z` is tiny.
//! 2. **The inverse family** (ARITH 2022 § IIB): a hard *output* first — a
//!    114-bit midpoint `z` per result binade and sign — then `x = round(2^z)`,
//!    whose `log2` lands within `2^-113·log2 e` of that midpoint.
//! 3. **Regression scan.** An MPFR near-midpoint scan, log-uniform over every
//!    binade and bit-uniform over the neighbourhood of 1 where the accurate
//!    leg decides alone, plus a plain random sample of the whole domain.
//!
//! Run with:
//! ```text
//! CC=clang cargo +nightly run --release --features "f128 mpfr" --example gen_f128_log2_cases
//! ```

use metallic::f128_mpfr::cr_unop;
use rug::Float;
use rug::float::Round;
use rug::ops::Pow;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// Working precision of the oracle: how far past the round bit the midpoint
/// distance is resolved.
const PREC: u32 = 250;

/// Keep scan results within this normalized distance of a binary128 midpoint
/// (0 a midpoint, 1 a grid point).
const THRESHOLD: f64 = 1.0 / 65_536.0;

/// Inputs scanned over the whole domain, and over the neighbourhood of 1.
const SCAN: u64 = 100_000_000;
const SCAN_NEAR_ONE: u64 = 50_000_000;

/// Hard outputs per result binade and sign.
const INVERSE: u64 = 32;

/// Plain random inputs kept over the whole domain, no filter.
const WIDE: u64 = 2_000;

const THREADS: u64 = 16;

const SIGN: u128 = 1 << 127;
const MANTISSA: u128 = (1 << 112) - 1;

fn mix(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn mix128(i: u64) -> u128 {
    u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9_7F4A_7C15)) << 64
}

/// A positive value with the exponent field uniform over every binade,
/// subnormals included: the whole domain of the logarithm.
fn positive(i: u64) -> f128 {
    let bits = mix128(i);
    let exponent = (bits >> 112) % 0x7fff;

    f128::from_bits(exponent << 112 | bits & MANTISSA)
}

/// `1 ± t` for `t` log-uniform over `[2^-114, 2^-1)`: the accurate leg's own
/// neighbourhood, where the reduction cancels.
fn near_one(i: u64) -> f128 {
    let bits = mix128(i);
    let exponent = 16383 - 114 + (bits >> 112 & 0x7fff) % 114;
    let t = f128::from_bits(exponent << 112 | bits & MANTISSA);

    if bits & SIGN == 0 { 1.0 + t } else { 1.0 - t }
}

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

fn log2(x: f128) -> f128 {
    cr_unop(x, |y| y.log2_round(Round::Nearest))
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

/// `2^k` as binary128, subnormals included.
fn power(k: i32) -> f128 {
    if k < -16382 {
        f128::from_bits(1 << (k + 16494))
    } else {
        f128::from_bits(((k + 16383) as u128) << 112)
    }
}

/// Layer 1: the edges.
fn edges() -> Vec<f128> {
    let step = |x: f128, k: i128| f128::from_bits((x.to_bits() as i128 + k) as u128);
    let mut out = vec![
        0.0,
        -0.0,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::NAN,
        -1.0,
    ];

    // The neighbours of 1, and of the fast leg's floor `|log2 x| = 2^-16`.
    out.extend((-4..=4).map(|d| step(1.0, d)));
    for floor in [
        Float::with_val(PREC, 2).pow(-16_i32).exp2(),
        (-Float::with_val(PREC, 2).pow(-16_i32)).exp2(),
    ] {
        let x = floor.to_f128_round(Round::Nearest);
        out.extend((-2..=2).map(|d| step(x, d)));
    }
    // Every power of two is exact; the neighbours of a sparse subset add a
    // 2^-112 fraction under an integer part as large as 2^14.
    for k in -16494..=16383 {
        let x = power(k);
        out.push(x);
        if k % 32 == 0 || [-16494, -16493, -16383, -16382, -16381, 16382, 16383].contains(&k) {
            out.extend([step(x, -1), step(x, 1)]);
        }
    }
    // Significands a few ulps from a table reciprocal `2^(j/2^18)`, so the
    // reduced `z` is under 2^-110 and the polynomial's squares are nearly
    // zero; `2 − 2^-k` sits within 2^-k of the top one.
    for i in 0..512_u64 {
        let bits = mix128(i);
        let j = (bits & 0x3ffff) as u32;
        let exponent = (bits >> 64) % 0x7ffe + 1;
        let m: Float = Float::with_val(PREC, j) / 262_144_u32;
        let m = m.exp2().to_f128_round(Round::Nearest).to_bits() & MANTISSA;
        let x = f128::from_bits(exponent << 112 | m);
        out.extend((-3..=3).map(|d| step(x, d)));
    }
    for k in 1..=112 {
        let m = (1_u128 << 112) - (1 << (112 - k));
        out.extend([0, 5000, 16382, 32765].map(|e| f128::from_bits(e << 112 | m)));
    }
    out
}

/// Layer 2: per result binade and sign, `x = round(2^z)` for a 114-bit
/// midpoint `z`, so that `log2 x` sits within `2^-113·log2 e` of it.
fn inverse() -> Vec<f128> {
    let mut out = Vec::new();

    for exponent in -20..=14_i32 {
        for sign in [1_i32, -1] {
            for i in 0..INVERSE {
                let odd = mix128((exponent as u64) << 40 | i << 8 | u64::from(sign < 0)) >> 15 | 1;
                let z: Float = Float::with_val(PREC, odd)
                    * sign
                    * Float::with_val(PREC, 2).pow(exponent - 113);
                out.push(z.exp2().to_f128_round(Round::Nearest));
            }
        }
    }
    out
}

/// Layer 3: the near-midpoint scan of `sampler`'s first `count` draws, in
/// parallel.
fn scan(sampler: fn(u64) -> f128, count: u64, label: &str) -> Vec<f128> {
    let done = AtomicU64::new(0);
    let mut survivors = std::thread::scope(|s| {
        let workers: Vec<_> = (0..THREADS)
            .map(|t| {
                let done = &done;
                s.spawn(move || {
                    let mut kept = Vec::new();
                    for i in (t..count).step_by(THREADS as usize) {
                        let x = sampler(i);
                        let mut y = Float::with_val(PREC, x);
                        y.log2_round(Round::Nearest);
                        if midpoint_frac(&y) < THRESHOLD {
                            kept.push(x);
                        }
                        if i % 10_000_000 < THREADS {
                            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                            eprintln!("{label}: {}%", n * 100 / (count / 10_000_000 * THREADS));
                        }
                    }
                    kept
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().expect("worker"))
            .collect::<Vec<_>>()
    });
    survivors.sort_by_key(|x| x.to_bits());
    survivors
}

fn main() {
    let edges = edges();
    let family = inverse();
    let wide: Vec<f128> = (0..WIDE).map(positive).collect();
    let domain = scan(positive, SCAN, "domain scan");
    let unit = scan(near_one, SCAN_NEAR_ONE, "near-1 scan");
    eprintln!(
        "{} edges, {} inverse, {} + {} near-midpoints",
        edges.len(),
        family.len(),
        domain.len(),
        unit.len()
    );

    let mut text = String::from(
        "# Hard-to-round cases for metallic::log2q(x), with their correctly rounded answers (MPFR).\n\
         # Generated by `CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_log2_cases`.\n",
    );
    for (title, inputs) in [
        (
            "special values, neighbours of 1 and of the fast floor, powers of two",
            &edges,
        ),
        (
            "round(2^z) for a 114-bit midpoint z, per result binade and sign",
            &family,
        ),
        ("random over the whole domain", &wide),
        (
            "near a rounding midpoint, from an MPFR scan over every binade",
            &domain,
        ),
        (
            "near a rounding midpoint, from an MPFR scan of 1 ± 2^-114..2^-1",
            &unit,
        ),
    ] {
        writeln!(text, "#\n# {title}\n#").unwrap();
        for &x in inputs {
            writeln!(text, "{} {}", hex(x), hex(log2(x))).unwrap();
        }
    }
    std::fs::write("tests/cases/log2q.wc", text).expect("write corpus");
    eprintln!("wrote tests/cases/log2q.wc");
}
