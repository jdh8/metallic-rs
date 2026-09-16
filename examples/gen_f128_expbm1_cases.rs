#![feature(f128)]
//! Generate answer-carrying exp2m1q/exp10m1q corpora. Three layers:
//! exact/midpoint cases and boundaries; inverse images of result midpoints
//! and continued fractions of the tiny slopes; independent MPFR scans.
//!
//! CC=clang cargo +nightly run --release --features "f128 mpfr" \
//!   --example gen_f128_expbm1_cases -- <2|10> [scan-count]

use metallic::f128_mpfr::cr_unop;
use rug::{Float, float::Round, ops::Pow};
use std::fmt::Write as _;

const PREC: u32 = 420;
const SIGN: u128 = 1 << 127;
const MASK: u128 = (1 << 112) - 1;

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}
fn bits(i: u64) -> u128 {
    u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9_7F4A_7C15)) << 64
}
fn power(e: i32) -> f128 {
    if e < -16382 {
        f128::from_bits(1 << (e + 16494))
    } else {
        f128::from_bits(((e + 16383) as u128) << 112)
    }
}
fn step(x: f128, d: i32) -> f128 {
    f128::from_bits(x.to_bits().wrapping_add_signed(d as i128))
}
fn neighbours(out: &mut Vec<f128>, x: f128, radius: i32) {
    out.extend((-radius..=radius).map(|d| step(x, d)));
}
fn evaluate(base: u32, y: &mut Float) -> std::cmp::Ordering {
    if base == 2 {
        y.exp2_m1_round(Round::Nearest)
    } else {
        y.exp10_m1_round(Round::Nearest)
    }
}
fn inverse_value(base: u32, z: Float) -> f128 {
    let x = if base == 2 { z.log2_1p() } else { z.log10_1p() };
    x.to_f128_round(Round::Nearest)
}
fn hex(x: f128) -> String {
    let b = x.to_bits();
    let sign = if b & SIGN == 0 { "" } else { "-" };
    let e = (b >> 112 & 0x7fff) as i32;
    let m = b & MASK;
    match (e, m) {
        (0x7fff, 0) => format!("{sign}inf"),
        (0x7fff, _) => format!("{sign}nan"),
        (0, 0) => format!("{sign}0x0p+0"),
        (0, _) => format!("{sign}0x0.{m:028x}p-16382"),
        _ => format!("{sign}0x1.{m:028x}p{:+}", e - 16383),
    }
}

fn edges(base: u32) -> Vec<f128> {
    let mut out = vec![
        0.0,
        -0.0,
        f128::NAN,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::MAX,
        -f128::MAX,
    ];
    // Every input power, including the subnormal ladder, and neighbours of
    // each seam and a sparse set throughout the representation.
    for e in -16494..=16383 {
        for x in [power(e), -power(e)] {
            out.push(x);
            if e % 128 == 0
                || [
                    -16382, -16381, -385, -384, -273, -272, -129, -128, -20, -19, -7, -6,
                ]
                .contains(&e)
            {
                neighbours(&mut out, x, 4);
            }
        }
    }
    for b in 1..=1024 {
        out.extend([f128::from_bits(b), -f128::from_bits(b)]);
    }
    // Rational values occur only at integral inputs: 2^x rational forces x
    // integral for dyadic x, and the factors 2 and 5 do the same for 10^x.
    // 2^k-1 is exact through k=113, a midpoint at 114; -1+2^-k is exact
    // through 113 and a midpoint at -114. 10^k-1 is exact through k=34;
    // larger integral k produces an odd integer too wide to be a midpoint.
    for k in -116..=512 {
        neighbours(&mut out, k as f128, 4);
    }
    let ln = Float::with_val(PREC, base).ln();
    let overflow: Float = Float::with_val(PREC, 2).pow(16384) - Float::with_val(PREC, 2).pow(16270);
    let cut = inverse_value(base, overflow);
    neighbours(&mut out, cut, 32);
    let minus_one = (Float::with_val(PREC, 2).ln() * -114_i32 / ln).to_f128_round(Round::Nearest);
    neighbours(&mut out, minus_one, 32);
    for sign in [1, -1] {
        let cut = inverse_value(base, Float::with_val(PREC, f128::MIN_POSITIVE) * sign);
        neighbours(&mut out, cut, 16);
    }
    // Exact dyadic table breakpoints and their neighbours on both signs.
    for i in 0..1024_u64 {
        let y = Float::with_val(PREC, (bits(i) & 0x3ffff) as u32) / 262144_u32;
        let x = if base == 2 {
            y
        } else {
            y / Float::with_val(PREC, 10).log2()
        };
        let x = x.to_f128_round(Round::Nearest);
        neighbours(&mut out, x, 2);
        neighbours(&mut out, -x, 2);
    }
    out
}

/// Result-first Diophantine family: round(log_b(1+z)) for binary128
/// midpoints z, including the subnormal grid and the saturation boundary.
/// Near zero this solves the linear slope/correction problem directly.
fn inverse(base: u32) -> Vec<f128> {
    let mut out = Vec::new();
    let exponents = (-400..=14)
        .chain(-16494..=-16381)
        .chain((-16380..-400).step_by(128))
        .chain((15..=16383).step_by(32));
    for e in exponents {
        let shift = (e - 113).max(-16495);
        let keep = (e - shift + 1) as u32;
        for i in 0..32_u64 {
            let odd = (bits((e as u64).wrapping_mul(256).wrapping_add(i)) >> (128 - keep))
                | 1
                | (1 << (keep - 1));
            for sign in [1, -1] {
                if sign < 0 && e >= 0 {
                    continue;
                }
                let z = Float::with_val(PREC, odd) * sign * Float::with_val(PREC, 2).pow(shift);
                let x = inverse_value(base, z);
                if x.is_finite() && x != 0.0 {
                    neighbours(&mut out, x, 1);
                }
            }
        }
    }
    out
}

/// Continued fractions of the irrational slope, independent of the random
/// scan. A convergent p/q with odd p puts q*ln(b) near a rounding midpoint.
/// Odd multiples move q into the 113-bit input binade while keeping p a
/// 114-bit output midpoint. The two normal output binades use different
/// powers of two; subnormals instead use the single fixed half-unit grid.
fn slope_family(base: u32) -> Vec<f128> {
    let mut out = Vec::new();
    let scales: &[i32] = if base == 2 { &[1, 2] } else { &[-1, 0, 1] };
    for &scale in scales {
        let mut a = Float::with_val(512, base).ln() * Float::with_val(512, 2).pow(scale);
        let (mut p0, mut p1, mut q0, mut q1) = (0_u128, 1_u128, 1_u128, 0_u128);
        loop {
            let k = a
                .to_integer_round(Round::Down)
                .unwrap()
                .0
                .to_u128()
                .unwrap();
            let top = if q1 == 0 {
                k
            } else {
                k.min(((1 << 113) - 1 - q0) / q1)
            };
            // Include intermediate convergents, especially the last ones
            // before the 113-bit denominator limit and odd numerators when
            // the principal convergent has an even numerator.
            let mut steps: Vec<u128> = (1..=top.min(64)).collect();
            if top > 64 {
                steps.extend([top - 1, top]);
            }
            for t in steps {
                let p = p0 + t * p1;
                let q = q0 + t * q1;
                if p & 1 == 0 || q < 1 << 56 {
                    continue;
                }
                if scale == 1 && q <= MASK {
                    for x in [f128::from_bits(q), -f128::from_bits(q)] {
                        neighbours(&mut out, x, 1);
                    }
                }
                let lo = (1_u128 << 112).div_ceil(q).max((1_u128 << 113).div_ceil(p));
                let hi = (((1_u128 << 113) - 1) / q).min(((1_u128 << 114) - 1) / p);
                for j in [lo | 1, hi.saturating_sub(1 - (hi & 1))] {
                    if j < lo || j > hi {
                        continue;
                    }
                    let m = q * j;
                    for e in [
                        -16382, -4096, -1000, -384, -300, -256, -227, -226, -225, -224,
                    ] {
                        let x = f128::from_bits(((16383 + e) as u128) << 112 | (m - (1 << 112)));
                        neighbours(&mut out, x, 1);
                        neighbours(&mut out, -x, 1);
                    }
                }
            }
            if top < k {
                break;
            }
            (p0, p1, q0, q1) = (p1, p0 + k * p1, q1, q0 + k * q1);
            a -= k;
            a.recip_mut();
        }
    }
    out
}

fn sample(i: u64, band: u32) -> f128 {
    let b = bits(i);
    let e = match band {
        0 => (b >> 112 & 0x7fff) as i32,
        1 => 16383 - 400 + (i % 415) as i32,
        2 => 16383 - 6 + (i % 21) as i32,
        _ => return f128::from_bits(b & SIGN | b & ((1 << 114) - 1)),
    };
    f128::from_bits(b & SIGN | (e as u128) << 112 | b & MASK)
}
fn midpoint_frac(y: &Float) -> f64 {
    if !y.is_finite() || y.is_zero() {
        return 1.0;
    }
    let y = Float::with_val(PREC, y.abs_ref());
    let rounded = y.to_f128_round(Round::Nearest);
    if !rounded.is_finite() || rounded == 0.0 {
        return 1.0;
    }
    let c = Float::with_val(PREC, rounded);
    let adjacent = if y > c {
        rounded.next_up()
    } else {
        rounded.next_down()
    };
    if !adjacent.is_finite() {
        return 1.0;
    }
    let gap = (Float::with_val(PREC, adjacent) - &c).abs();
    let distance = (y - c).abs() / gap;
    (1.0 - 2.0 * distance.to_f64()).abs()
}
fn scan(base: u32, band: u32, count: u64) -> Vec<f128> {
    let mut out = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..4)
            .map(|t| {
                scope.spawn(move || {
                    let mut kept = Vec::new();
                    for i in (t..count).step_by(4) {
                        let x = sample(i, band);
                        let mut y = Float::with_val(PREC, x);
                        evaluate(base, &mut y);
                        if midpoint_frac(&y) < 1.0 / 65536.0 {
                            kept.push(x);
                        }
                    }
                    kept
                })
            })
            .collect();
        jobs.into_iter()
            .flat_map(|j| j.join().unwrap())
            .collect::<Vec<_>>()
    });
    out.sort_by_key(|x| x.to_bits());
    eprintln!("band {band}: kept {} from {count}", out.len());
    out
}
fn main() {
    let base: u32 = std::env::args()
        .nth(1)
        .expect("base 2 or 10")
        .parse()
        .unwrap();
    assert!([2, 10].contains(&base));
    let count: u64 = std::env::args()
        .nth(2)
        .map_or(4_000_000, |s| s.parse().unwrap());
    let mut sections = vec![
        (
            "special, exact, midpoint, thresholds and seams".to_owned(),
            edges(base),
        ),
        (
            "inverse images of binary128 midpoints".to_owned(),
            inverse(base),
        ),
        (
            "continued fractions of ln(base), normal and subnormal midpoint grids".to_owned(),
            slope_family(base),
        ),
        (
            "unfiltered deterministic representation sample".to_owned(),
            (0..4000).map(|i| sample(i, 0)).collect(),
        ),
    ];
    for band in 0..4 {
        sections.push((
            format!("MPFR near-midpoint scan, band {band}, {count} samples"),
            scan(base, band, count),
        ));
    }
    let mut text = format!(
        "# exp{base}m1q inputs and correctly rounded MPFR answers.\n# Generated by CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_expbm1_cases -- {base} {count}\n"
    );
    let mut total = 0;
    for (title, inputs) in sections {
        writeln!(text, "#\n# {title}").unwrap();
        total += inputs.len();
        for x in inputs {
            let y = cr_unop(x, |y| evaluate(base, y));
            writeln!(text, "{} {}", hex(x), hex(y)).unwrap();
        }
    }
    let path = format!("tests/cases/exp{base}m1q.wc");
    std::fs::write(&path, text).unwrap();
    eprintln!("wrote {total} cases to {path}");
}
